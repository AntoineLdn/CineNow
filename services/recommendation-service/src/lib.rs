use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use shared::{compute_affinity_score, Movie};

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

pub fn compute_recommendations(
    moods: &[String],
    weather: &WeatherData,
    movies_by_genre: &HashMap<String, Vec<Movie>>,
) -> Vec<RecommendedMovie> {
    // Dédupliquer les films (un même film peut apparaître dans plusieurs genres)
    let mut all_movies: HashMap<u32, &Movie> = HashMap::new();
    for films in movies_by_genre.values() {
        for film in films {
            all_movies.entry(film.id).or_insert(film);
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

    fn make_movie(id: u32, genres: &[&str], rating: f32) -> Movie {
        Movie {
            id,
            title: format!("Film {}", id),
            genres: genres.iter().map(|g| g.to_string()).collect(),
            rating,
            poster_path: None,
            overview: String::new(),
            release_date: None,
        }
    }

    fn make_catalog() -> HashMap<String, Vec<Movie>> {
        let mut catalog = HashMap::new();
        catalog.insert("comedie".to_string(), vec![
            make_movie(1, &["comedie"], 8.0),
            make_movie(2, &["comedie", "romance"], 7.5),
        ]);
        catalog.insert("action".to_string(), vec![
            make_movie(3, &["action", "thriller"], 8.5),
            make_movie(4, &["action"], 7.0),
        ]);
        catalog.insert("drame".to_string(), vec![
            make_movie(5, &["drame"], 9.0),
            make_movie(6, &["drame", "histoire"], 6.5),
        ]);
        catalog
    }

    #[test]
    fn test_joy_soleil_comedie_en_tete() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Ensoleillé".to_string(),
            period: "Matin".to_string(),
        };

        let result = compute_recommendations(&["joy".to_string()], &weather, &catalog);

        assert!(!result.is_empty());
        // Le film 1 (comedie pure, rating 8.0) doit avoir un meilleur score que le film 5 (drame, rating 9.0)
        let score_comedie = result.iter().find(|r| r.id == 1).map(|r| r.score).unwrap_or(0.0);
        let score_drame = result.iter().find(|r| r.id == 5).map(|r| r.score).unwrap_or(0.0);
        assert!(score_comedie > score_drame,
            "joy+soleil: comédie (score={:.3}) devrait battre drame (score={:.3})", score_comedie, score_drame);
    }

    #[test]
    fn test_angry_orageux_action_en_tete() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Orageux".to_string(),
            period: "Soir".to_string(),
        };

        let result = compute_recommendations(&["angry".to_string()], &weather, &catalog);

        assert!(!result.is_empty());
        let score_action = result.iter().find(|r| r.id == 3).map(|r| r.score).unwrap_or(0.0);
        let score_comedie = result.iter().find(|r| r.id == 1).map(|r| r.score).unwrap_or(0.0);
        assert!(score_action > score_comedie,
            "angry+orage: action (score={:.3}) devrait battre comedie (score={:.3})", score_action, score_comedie);
    }

    #[test]
    fn test_multi_genre_booste() {
        let catalog = make_catalog();
        let weather = WeatherData {
            condition: "Ensoleillé".to_string(),
            period: "Matin".to_string(),
        };

        let result = compute_recommendations(&["joy".to_string()], &weather, &catalog);

        // Film 2 a comedie + romance, les deux boostés par joy+soleil
        // Il doit avoir un meilleur score que film 4 (action pure, hors contexte)
        let score_multi = result.iter().find(|r| r.id == 2).map(|r| r.score).unwrap_or(0.0);
        let score_action = result.iter().find(|r| r.id == 4).map(|r| r.score).unwrap_or(0.0);
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
        );

        let ids: Vec<u32> = result.iter().map(|r| r.id).collect();
        let has_comedie = ids.iter().any(|id| [1, 2].contains(id));
        let has_drame = ids.iter().any(|id| [5, 6].contains(id));
        assert!(has_comedie && has_drame,
            "Multi-humeurs devrait inclure comédie et drame, ids: {:?}", ids);
    }

    #[test]
    fn test_max_4_par_genre() {
        let mut catalog = HashMap::new();
        catalog.insert("action".to_string(),
            (1u32..=6).map(|i| make_movie(i, &["action"], 8.0)).collect()
        );
        catalog.insert("comedie".to_string(), vec![make_movie(10, &["comedie"], 7.0)]);

        let weather = WeatherData {
            condition: "Orageux".to_string(),
            period: "Nuit".to_string(),
        };

        let result = compute_recommendations(&["angry".to_string()], &weather, &catalog);

        let action_count = result.iter().filter(|r| r.id >= 1 && r.id <= 6).count();
        assert!(action_count <= 4,
            "Max 4 films par genre, mais on a {} films d'action", action_count);
    }

    #[test]
    fn test_catalogue_vide() {
        let result = compute_recommendations(
            &["joy".to_string()],
            &WeatherData { condition: "Ensoleillé".to_string(), period: "Matin".to_string() },
            &HashMap::new(),
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

        let result = compute_recommendations(&["joy".to_string()], &weather, &catalog);

        // Tous les scores doivent être entre 0.0 et 1.0
        for r in &result {
            assert!(r.score >= 0.0 && r.score <= 1.0,
                "Score hors bornes pour film {} : {}", r.id, r.score);
        }
    }

    #[test]
    fn test_deduplication() {
        // Même film dans deux genres différents → apparaît une seule fois dans les résultats
        let mut catalog = HashMap::new();
        catalog.insert("action".to_string(), vec![make_movie(1, &["action", "thriller"], 8.0)]);
        catalog.insert("thriller".to_string(), vec![make_movie(1, &["action", "thriller"], 8.0)]);

        let weather = WeatherData {
            condition: "Orageux".to_string(),
            period: "Soir".to_string(),
        };

        let result = compute_recommendations(&["angry".to_string()], &weather, &catalog);
        let count = result.iter().filter(|r| r.id == 1).count();
        assert_eq!(count, 1, "Film dupliqué dans le catalogue → doit apparaître une seule fois");
    }
}