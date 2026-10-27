use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use shared::{compute_affinity_score, Movie};
use std::collections::HashMap;

/// En dessous, la note n'est pas assez étayée pour départager quoi que ce soit :
/// un film obscur à 10/10 sur 2 votes passerait devant tout le monde.
///
/// La valeur vient de la répartition réelle des `vote_count` du catalogue, qui
/// est nettement bimodale (p10 ≈ 7 votes, p25 ≈ 256) : tout seuil entre 25 et
/// 500 écarte à peu près la même population, 50 tombe au milieu du creux.
pub const MIN_VOTES_FOR_RECOMMENDATION: u32 = 50;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WeatherData {
    pub condition: String,
    pub period: String,
}

#[derive(Debug, Serialize)]
pub struct RecommendedMovie {
    pub id: u32,
    pub score: f32,
}

/// Film proposable ce soir : déjà sorti, et noté par assez de monde pour que la
/// note veuille dire quelque chose.
///
/// Une date de sortie absente ou illisible exclut le film : on ne peut pas
/// affirmer qu'il est sorti, et « Ce soir » ne doit jamais proposer autre chose.
fn is_recommendable(movie: &Movie, today: NaiveDate) -> bool {
    if movie.rating.is_none() || movie.vote_count < MIN_VOTES_FOR_RECOMMENDATION {
        return false;
    }

    match movie.release_date.as_deref() {
        Some(date) => match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            Ok(release) => release <= today,
            Err(_) => false,
        },
        None => false,
    }
}

/// `today` est passé en paramètre plutôt que lu dans la fonction, pour que les
/// tests soient déterministes.
pub fn compute_recommendations(
    moods: &[String],
    weather: &WeatherData,
    movies_by_genre: &HashMap<String, Vec<Movie>>,
    today: NaiveDate,
) -> Vec<RecommendedMovie> {
    // Dédupliquer les films (un même film peut apparaître dans plusieurs genres)
    let mut all_movies: HashMap<u32, &Movie> = HashMap::new();
    for films in movies_by_genre.values() {
        for film in films {
            if is_recommendable(film, today) {
                all_movies.entry(film.id).or_insert(film);
            }
        }
    }

    // Calculer le score pour chaque film sur chaque humeur, garder le meilleur
    let mut scores: Vec<(u32, f32)> = all_movies
        .values()
        .map(|film| {
            let best_score = moods
                .iter()
                .map(|mood| compute_affinity_score(film, mood, &weather.condition))
                .fold(0.0_f32, f32::max);

            (film.id, best_score)
        })
        .collect();

    // Trier par score décroissant
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Diversifier : max 4 films par genre, top 20
    let mut genre_count: HashMap<String, usize> = HashMap::new();
    let mut result: Vec<RecommendedMovie> = Vec::new();

    for (id, score) in scores {
        if result.len() >= 20 {
            break;
        }

        let film = all_movies[&id];

        // Genre dominant = premier genre du film
        let dominant_genre = film.genres.first().cloned().unwrap_or_default();
        let count = genre_count.entry(dominant_genre).or_insert(0);

        if *count < 4 {
            *count += 1;
            result.push(RecommendedMovie { id, score });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Date de référence : les films du catalogue de test sortent en 2000, donc
    // tous sortis, sauf là où un test en décide autrement.
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()
    }

    fn make_movie(id: u32, genres: &[&str], rating: f32) -> Movie {
        Movie {
            id,
            title: format!("Film {}", id),
            genres: genres.iter().map(|g| g.to_string()).collect(),
            rating: Some(rating),
            vote_count: 1200,
            poster_path: None,
            overview: String::new(),
            release_date: Some("2000-01-01".to_string()),
        }
    }

    fn make_catalog() -> HashMap<String, Vec<Movie>> {
        let mut catalog = HashMap::new();
        catalog.insert(
            "comedie".to_string(),
            vec![
                make_movie(1, &["comedie"], 8.0),
                make_movie(2, &["comedie", "romance"], 7.5),
            ],
        );
        catalog.insert(
            "action".to_string(),
            vec![
                make_movie(3, &["action", "thriller"], 8.5),
                make_movie(4, &["action"], 7.0),
            ],
        );
        catalog.insert(
            "drame".to_string(),
            vec![
                make_movie(5, &["drame"], 9.0),
                make_movie(6, &["drame", "histoire"], 6.5),
            ],
        );
        catalog
    }

    // ----------------------------------------------- Critères de recommandation

    fn single(rating: Option<f32>, vote_count: u32, release_date: Option<&str>) -> Movie {
        Movie {
            id: 1,
            title: "Film".to_string(),
            genres: vec!["comedie".to_string()],
            rating,
            vote_count,
            poster_path: None,
            overview: String::new(),
            release_date: release_date.map(str::to_string),
        }
    }

    fn recommend(movie: Movie) -> Vec<RecommendedMovie> {
        let catalog = HashMap::from([("comedie".to_string(), vec![movie])]);
        let weather = WeatherData {
            condition: "Ensoleillé".to_string(),
            period: "Matin".to_string(),
        };
        compute_recommendations(&["joy".to_string()], &weather, &catalog, today())
    }

    #[test]
    fn exclut_un_film_pas_encore_sorti() {
        let demain = single(Some(8.0), 1200, Some("2026-06-16"));
        assert!(recommend(demain).is_empty());
    }

    #[test]
    fn inclut_un_film_sortant_aujourd_hui() {
        let aujourdhui = single(Some(8.0), 1200, Some("2026-06-15"));
        assert_eq!(recommend(aujourdhui).len(), 1);
    }

    #[test]
    fn exclut_un_film_sans_note() {
        let sans_note = single(None, 0, Some("2000-01-01"));
        assert!(recommend(sans_note).is_empty());
    }

    #[test]
    fn exclut_un_film_sous_le_seuil_de_votes() {
        let sous_le_seuil = single(
            Some(10.0),
            MIN_VOTES_FOR_RECOMMENDATION - 1,
            Some("2000-01-01"),
        );
        assert!(
            recommend(sous_le_seuil).is_empty(),
            "un 10/10 sur trop peu de votes ne doit pas remonter"
        );

        let au_seuil = single(Some(10.0), MIN_VOTES_FOR_RECOMMENDATION, Some("2000-01-01"));
        assert_eq!(recommend(au_seuil).len(), 1, "le seuil lui-même est inclus");
    }

    #[test]
    fn exclut_un_film_sans_date_de_sortie() {
        assert!(recommend(single(Some(8.0), 1200, None)).is_empty());
    }

    #[test]
    fn exclut_une_date_de_sortie_illisible() {
        assert!(recommend(single(Some(8.0), 1200, Some("bientôt"))).is_empty());
    }

    #[test]
    fn test_joy_soleil_comedie_en_tete() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Ensoleillé".to_string(),
            period: "Matin".to_string(),
        };

        let result = compute_recommendations(&["joy".to_string()], &weather, &catalog, today());

        assert!(!result.is_empty());
        // Le film 1 (comedie pure, rating 8.0) doit avoir un meilleur score que le film 5 (drame, rating 9.0)
        let score_comedie = result
            .iter()
            .find(|r| r.id == 1)
            .map(|r| r.score)
            .unwrap_or(0.0);
        let score_drame = result
            .iter()
            .find(|r| r.id == 5)
            .map(|r| r.score)
            .unwrap_or(0.0);
        assert!(
            score_comedie > score_drame,
            "joy+soleil: comédie (score={:.3}) devrait battre drame (score={:.3})",
            score_comedie,
            score_drame
        );
    }

    #[test]
    fn test_angry_orageux_action_en_tete() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Orageux".to_string(),
            period: "Soir".to_string(),
        };

        let result = compute_recommendations(&["angry".to_string()], &weather, &catalog, today());

        assert!(!result.is_empty());
        let score_action = result
            .iter()
            .find(|r| r.id == 3)
            .map(|r| r.score)
            .unwrap_or(0.0);
        let score_comedie = result
            .iter()
            .find(|r| r.id == 1)
            .map(|r| r.score)
            .unwrap_or(0.0);
        assert!(
            score_action > score_comedie,
            "angry+orage: action (score={:.3}) devrait battre comedie (score={:.3})",
            score_action,
            score_comedie
        );
    }

    #[test]
    fn test_multi_genre_booste() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Ensoleillé".to_string(),
            period: "Matin".to_string(),
        };

        let result = compute_recommendations(&["joy".to_string()], &weather, &catalog, today());

        // Film 2 a comedie + romance, les deux boostés par joy+soleil
        // Il doit avoir un meilleur score que film 4 (action pure, hors contexte)
        let score_multi = result
            .iter()
            .find(|r| r.id == 2)
            .map(|r| r.score)
            .unwrap_or(0.0);
        let score_action = result
            .iter()
            .find(|r| r.id == 4)
            .map(|r| r.score)
            .unwrap_or(0.0);
        assert!(score_multi > score_action,
            "Film multi-genre booste (score={:.3}) devrait battre action hors contexte (score={:.3})",
            score_multi, score_action);
    }

    #[test]
    fn test_multi_humeurs() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Nuageux".to_string(),
            period: "Après-midi".to_string(),
        };

        let result = compute_recommendations(
            &["joy".to_string(), "sad".to_string()],
            &weather,
            &catalog,
            today(),
        );

        let ids: Vec<u32> = result.iter().map(|r| r.id).collect();
        let has_comedie = ids.iter().any(|id| [1, 2].contains(id));
        let has_drame = ids.iter().any(|id| [5, 6].contains(id));
        assert!(
            has_comedie && has_drame,
            "Multi-humeurs devrait inclure comédie et drame, ids: {:?}",
            ids
        );
    }

    #[test]
    fn test_max_4_par_genre() {
        let mut catalog = HashMap::new();
        catalog.insert(
            "action".to_string(),
            (1u32..=6)
                .map(|i| make_movie(i, &["action"], 8.0))
                .collect(),
        );
        catalog.insert(
            "comedie".to_string(),
            vec![make_movie(10, &["comedie"], 7.0)],
        );

        let weather = WeatherData {
            condition: "Orageux".to_string(),
            period: "Nuit".to_string(),
        };

        let result = compute_recommendations(&["angry".to_string()], &weather, &catalog, today());

        let action_count = result.iter().filter(|r| r.id >= 1 && r.id <= 6).count();
        assert!(
            action_count <= 4,
            "Max 4 films par genre, mais on a {} films d'action",
            action_count
        );
    }

    #[test]
    fn test_catalogue_vide() {
        let result = compute_recommendations(
            &["joy".to_string()],
            &WeatherData {
                condition: "Ensoleillé".to_string(),
                period: "Matin".to_string(),
            },
            &HashMap::new(),
            today(),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn test_scores_normalises() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Ensoleillé".to_string(),
            period: "Matin".to_string(),
        };

        let result = compute_recommendations(&["joy".to_string()], &weather, &catalog, today());

        // Tous les scores doivent être entre 0.0 et 1.0
        for r in &result {
            assert!(
                r.score >= 0.0 && r.score <= 1.0,
                "Score hors bornes pour film {} : {}",
                r.id,
                r.score
            );
        }
    }

    #[test]
    fn test_deduplication() {
        // Même film dans deux genres différents → apparaît une seule fois dans les résultats
        let mut catalog = HashMap::new();
        catalog.insert(
            "action".to_string(),
            vec![make_movie(1, &["action", "thriller"], 8.0)],
        );
        catalog.insert(
            "thriller".to_string(),
            vec![make_movie(1, &["action", "thriller"], 8.0)],
        );

        let weather = WeatherData {
            condition: "Orageux".to_string(),
            period: "Soir".to_string(),
        };

        let result = compute_recommendations(&["angry".to_string()], &weather, &catalog, today());
        let count = result.iter().filter(|r| r.id == 1).count();
        assert_eq!(
            count, 1,
            "Film dupliqué dans le catalogue → doit apparaître une seule fois"
        );
    }
}
