use shared::{rating_from_votes, tmdb_genre_ids_to_names, Movie};
use std::collections::HashMap;
use tokio::task::JoinSet;

/// Au-delà, la requête part en autant d'appels TMDb : on refuse plutôt que de
/// laisser un client saturer le service.
pub const MAX_IDS: usize = 100;

#[derive(Debug, PartialEq, Eq)]
pub enum IdsError {
    Empty,
    TooMany(usize),
    Invalid(String),
}

impl IdsError {
    pub fn message(&self) -> String {
        match self {
            IdsError::Empty => "Aucun identifiant fourni".to_string(),
            IdsError::TooMany(count) => {
                format!("{count} identifiants demandés, maximum {MAX_IDS}")
            }
            IdsError::Invalid(segment) => format!("Identifiant invalide : {segment}"),
        }
    }
}

/// Analyse le paramètre `ids=1,2,3`.
///
/// Les doublons sont retirés en conservant la première position : demander deux
/// fois le même film ne doit pas déclencher deux appels TMDb.
pub fn parse_ids(raw: &str) -> Result<Vec<u32>, IdsError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(IdsError::Empty);
    }

    let segments: Vec<&str> = trimmed.split(',').collect();
    if segments.len() > MAX_IDS {
        return Err(IdsError::TooMany(segments.len()));
    }

    let mut ids = Vec::with_capacity(segments.len());
    for segment in segments {
        let candidate = segment.trim();
        let id: u32 = candidate
            .parse()
            .map_err(|_| IdsError::Invalid(candidate.to_string()))?;

        if !ids.contains(&id) {
            ids.push(id);
        }
    }

    Ok(ids)
}

/// Assemble les films demandés dans l'ordre des ids.
///
/// `known` contient ce que le catalogue MQTT sait déjà ; le reste est récupéré
/// sur TMDb en parallèle. Les ids que TMDb ignore disparaissent simplement du
/// résultat, sans erreur.
pub async fn resolve(mut known: HashMap<u32, Movie>, ids: &[u32], tmdb_key: &str) -> Vec<Movie> {
    let missing: Vec<u32> = ids
        .iter()
        .copied()
        .filter(|id| !known.contains_key(id))
        .collect();

    if !missing.is_empty() && !tmdb_key.is_empty() {
        let client = reqwest::Client::new();
        let mut fetches = JoinSet::new();

        for id in missing {
            let client = client.clone();
            let key = tmdb_key.to_string();
            fetches.spawn(async move { fetch_from_tmdb(&client, &key, id).await });
        }

        while let Some(finished) = fetches.join_next().await {
            if let Ok(Some(movie)) = finished {
                known.insert(movie.id, movie);
            }
        }
    }

    ids.iter().filter_map(|id| known.remove(id)).collect()
}

/// Langue demandée à TMDb, puis langues de repli pour les champs traduits.
///
/// Beaucoup de films n'ont pas de traduction française : TMDb renvoie alors un
/// champ vide plutôt qu'une absence, d'où le repli.
pub const PRIMARY_LANGUAGE: &str = "fr-FR";
pub const FALLBACK_LANGUAGES: &[&str] = &["en-US"];

/// Fiche TMDb d'un film dans une langue donnée.
pub async fn fetch_details(
    client: &reqwest::Client,
    tmdb_key: &str,
    movie_id: u32,
    language: &str,
) -> Option<serde_json::Value> {
    let url = format!(
        "https://api.themoviedb.org/3/movie/{movie_id}?api_key={tmdb_key}&language={language}"
    );
    client.get(&url).send().await.ok()?.json().await.ok()
}

/// Premier texte non vide pour `field`, en essayant les langues dans l'ordre.
///
/// `primary` est la réponse déjà en main dans la langue principale : on ne la
/// redemande pas, et on n'interroge une langue de repli que tant que le champ
/// reste vide. Générique sur le champ et sur la liste de langues, pour servir
/// aux autres textes traduits (titre, accroche) le jour venu.
pub async fn text_with_fallback(
    client: &reqwest::Client,
    tmdb_key: &str,
    movie_id: u32,
    field: &str,
    primary: &serde_json::Value,
    fallbacks: &[&str],
) -> String {
    let from_primary = primary[field].as_str().unwrap_or_default();
    if !from_primary.is_empty() {
        return from_primary.to_string();
    }

    for language in fallbacks {
        let Some(details) = fetch_details(client, tmdb_key, movie_id, language).await else {
            continue;
        };
        let text = details[field].as_str().unwrap_or_default();
        if !text.is_empty() {
            return text.to_string();
        }
    }

    String::new()
}

/// Récupère un film sur TMDb. `None` si l'id est inconnu ou l'appel en échec.
async fn fetch_from_tmdb(client: &reqwest::Client, tmdb_key: &str, id: u32) -> Option<Movie> {
    let url = format!(
        "https://api.themoviedb.org/3/movie/{id}?api_key={tmdb_key}&language={PRIMARY_LANGUAGE}"
    );

    let payload: serde_json::Value = client.get(&url).send().await.ok()?.json().await.ok()?;

    // Un id inconnu renvoie un corps d'erreur, sans champ `id`.
    let id = payload["id"].as_u64()? as u32;

    // Les genres sont convertis depuis leurs ids, comme pour le catalogue :
    // passer par les noms TMDb donnerait « Science-Fiction » au lieu de
    // « science-fiction ».
    let genre_ids: Vec<u64> = payload["genres"]
        .as_array()
        .map(|genres| genres.iter().filter_map(|g| g["id"].as_u64()).collect())
        .unwrap_or_default();

    let vote_count = payload["vote_count"].as_u64().unwrap_or(0) as u32;

    Some(Movie {
        id,
        title: payload["title"].as_str().unwrap_or_default().to_string(),
        genres: tmdb_genre_ids_to_names(&genre_ids),
        rating: rating_from_votes(payload["vote_average"].as_f64().unwrap_or(0.0), vote_count),
        vote_count,
        poster_path: payload["poster_path"].as_str().map(str::to_string),
        overview: payload["overview"].as_str().unwrap_or_default().to_string(),
        release_date: payload["release_date"].as_str().map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyse_une_liste_simple() {
        assert_eq!(parse_ids("1,2,3"), Ok(vec![1, 2, 3]));
    }

    #[test]
    fn tolere_les_espaces() {
        assert_eq!(parse_ids(" 1 , 2 ,3 "), Ok(vec![1, 2, 3]));
    }

    #[test]
    fn conserve_l_ordre_demande() {
        assert_eq!(parse_ids("680,550,13"), Ok(vec![680, 550, 13]));
    }

    #[test]
    fn retire_les_doublons() {
        assert_eq!(parse_ids("5,5,7,5"), Ok(vec![5, 7]));
    }

    #[test]
    fn refuse_une_liste_vide() {
        assert_eq!(parse_ids(""), Err(IdsError::Empty));
        assert_eq!(parse_ids("   "), Err(IdsError::Empty));
    }

    #[test]
    fn refuse_un_segment_vide() {
        assert_eq!(parse_ids("1,,2"), Err(IdsError::Invalid(String::new())));
    }

    #[test]
    fn refuse_ce_qui_n_est_pas_un_nombre() {
        assert_eq!(
            parse_ids("1,abc"),
            Err(IdsError::Invalid("abc".to_string()))
        );
        assert_eq!(parse_ids("-1"), Err(IdsError::Invalid("-1".to_string())));
        assert_eq!(parse_ids("1.5"), Err(IdsError::Invalid("1.5".to_string())));
    }

    #[test]
    fn refuse_au_dela_de_la_limite() {
        let trop = (1..=MAX_IDS + 1)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");

        assert_eq!(parse_ids(&trop), Err(IdsError::TooMany(MAX_IDS + 1)));
    }

    #[test]
    fn accepte_la_limite_exacte() {
        let juste = (1..=MAX_IDS)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");

        assert_eq!(parse_ids(&juste).map(|ids| ids.len()), Ok(MAX_IDS));
    }

    fn movie(id: u32) -> Movie {
        Movie {
            id,
            title: format!("Film {id}"),
            genres: vec!["drame".to_string()],
            rating: Some(7.0),
            vote_count: 1200,
            poster_path: None,
            overview: String::new(),
            release_date: None,
        }
    }

    #[tokio::test]
    async fn rend_le_catalogue_dans_l_ordre_demande() {
        let known = HashMap::from([(1, movie(1)), (2, movie(2)), (3, movie(3))]);

        // Clé TMDb vide : aucun appel réseau, les absents restent absents.
        let resolved = resolve(known, &[3, 1, 2], "").await;

        assert_eq!(
            resolved.iter().map(|m| m.id).collect::<Vec<_>>(),
            vec![3, 1, 2]
        );
    }

    #[tokio::test]
    async fn ignore_les_ids_introuvables() {
        let known = HashMap::from([(1, movie(1))]);

        let resolved = resolve(known, &[1, 999], "").await;

        assert_eq!(resolved.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1]);
    }
}
