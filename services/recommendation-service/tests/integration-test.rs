use std::collections::HashMap;
use recommendation_service::{WeatherData, compute_recommendations};

fn make_movie(id: u32, genres: &[&str], rating: f32) -> shared::Movie {
    shared::Movie {
        id,
        title: format!("Film {}", id),
        genres: genres.iter().map(|g| g.to_string()).collect(),
        rating,
        poster_path: None,
        overview: String::new(),
        release_date: None,
    }
}

fn make_catalog() -> HashMap<String, Vec<shared::Movie>> {
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
    catalog.insert("romance".to_string(), vec![
        make_movie(7, &["romance"], 7.8),
    ]);
    catalog.insert("thriller".to_string(), vec![
        make_movie(8, &["thriller"], 8.2),
    ]);
    catalog
}

/// Scénario 1 : joy + soleil → comédie/romance dominent
#[test]
fn test_joy_soleil_comedie_en_tete() {
    let catalog = make_catalog();
    let weather = WeatherData {
        condition: "Ensoleillé".to_string(),
        period: "Matin".to_string(),
    };

    let result = compute_recommendations(&["joy".to_string()], &weather, &catalog);

    assert!(!result.is_empty());
    let score_comedie = result.iter().find(|r| r.id == 1).map(|r| r.score).unwrap_or(0.0);
    let score_drame   = result.iter().find(|r| r.id == 5).map(|r| r.score).unwrap_or(0.0);
    assert!(score_comedie > score_drame,
        "joy+soleil: comédie (score={:.3}) devrait battre drame (score={:.3})",
        score_comedie, score_drame);
}

/// Scénario 2 : angry + orage → action/thriller dominent
#[test]
fn test_angry_orageux_action_en_tete() {
    let catalog = make_catalog();
    let weather = WeatherData {
        condition: "Orageux".to_string(),
        period: "Soir".to_string(),
    };

    let result = compute_recommendations(&["angry".to_string()], &weather, &catalog);

    assert!(!result.is_empty());
    let score_action  = result.iter().find(|r| r.id == 3).map(|r| r.score).unwrap_or(0.0);
    let score_comedie = result.iter().find(|r| r.id == 1).map(|r| r.score).unwrap_or(0.0);
    assert!(score_action > score_comedie,
        "angry+orage: action (score={:.3}) devrait battre comedie (score={:.3})",
        score_action, score_comedie);
}

/// Scénario 3 : film multi-genre bénéficie de plusieurs boosts
#[test]
fn test_multi_genre_booste() {
    let catalog = make_catalog();
    let weather = WeatherData {
        condition: "Ensoleillé".to_string(),
        period: "Matin".to_string(),
    };

    let result = compute_recommendations(&["joy".to_string()], &weather, &catalog);

    // Film 2 (comedie + romance) vs film 4 (action seul, hors contexte)
    let score_multi  = result.iter().find(|r| r.id == 2).map(|r| r.score).unwrap_or(0.0);
    let score_action = result.iter().find(|r| r.id == 4).map(|r| r.score).unwrap_or(0.0);
    assert!(score_multi > score_action,
        "Film multi-genre booste (score={:.3}) devrait battre action hors contexte (score={:.3})",
        score_multi, score_action);
}

/// Scénario 4 : multi-humeurs → diversité comédie + drame présents
#[test]
fn test_multi_humeurs_diversite() {
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
    let has_drame   = ids.iter().any(|id| [5, 6].contains(id));
    assert!(has_comedie && has_drame,
        "Multi-humeurs devrait inclure comédie et drame, ids: {:?}", ids);
}

/// Scénario 5 : max 4 films par genre respecté
#[test]
fn test_max_4_par_genre() {
    let mut catalog = HashMap::new();
    catalog.insert("action".to_string(),
        (1u32..=6).map(|i| make_movie(i, &["action"], 8.0)).collect()
    );
    catalog.insert("comedie".to_string(), vec![
        make_movie(10, &["comedie"], 7.0),
    ]);

    let weather = WeatherData {
        condition: "Orageux".to_string(),
        period: "Nuit".to_string(),
    };

    let result = compute_recommendations(&["angry".to_string()], &weather, &catalog);

    let action_count = result.iter().filter(|r| r.id >= 1 && r.id <= 6).count();
    assert!(action_count <= 4,
        "Max 4 films par genre, mais on a {} films d'action", action_count);
}

/// Scénario 6 : catalogue vide → résultat vide sans panique
#[test]
fn test_catalogue_vide() {
    let result = compute_recommendations(
        &["joy".to_string()],
        &WeatherData { condition: "Ensoleillé".to_string(), period: "Matin".to_string() },
        &HashMap::new(),
    );
    assert!(result.is_empty());
}

/// Scénario 7 : scores normalisés entre 0.0 et 1.0
#[test]
fn test_scores_normalises() {
    let catalog = make_catalog();
    let weather = WeatherData {
        condition: "Ensoleillé".to_string(),
        period: "Matin".to_string(),
    };

    let result = compute_recommendations(&["joy".to_string()], &weather, &catalog);

    for r in &result {
        assert!(r.score >= 0.0 && r.score <= 1.0,
            "Score hors bornes pour film {} : {}", r.id, r.score);
    }
}

/// Scénario 8 : film présent dans deux genres → apparaît une seule fois
#[test]
fn test_deduplication() {
    let mut catalog = HashMap::new();
    catalog.insert("action".to_string(),   vec![make_movie(1, &["action", "thriller"], 8.0)]);
    catalog.insert("thriller".to_string(), vec![make_movie(1, &["action", "thriller"], 8.0)]);

    let weather = WeatherData {
        condition: "Orageux".to_string(),
        period: "Soir".to_string(),
    };

    let result = compute_recommendations(&["angry".to_string()], &weather, &catalog);
    let count = result.iter().filter(|r| r.id == 1).count();
    assert_eq!(count, 1, "Film dupliqué dans le catalogue → doit apparaître une seule fois");
}

/// Scénario 9 : humeur inconnue → résultat non vide, scores valides
#[test]
fn test_humeur_inconnue() {
    let catalog = make_catalog();
    let weather = WeatherData {
        condition: "Pluie".to_string(),
        period: "Soir".to_string(),
    };

    let result = compute_recommendations(&["inconnu".to_string()], &weather, &catalog);

    assert!(!result.is_empty());
    for r in &result {
        assert!(r.score >= 0.0 && r.score <= 1.0,
            "Score hors bornes pour humeur inconnue, film {} : {}", r.id, r.score);
    }
}