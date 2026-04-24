use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: u32,
    pub title: String,
    pub genres: Vec<String>,
    pub rating: f32,
    pub poster_path: Option<String>,
    pub overview: String,
    pub release_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieCatalog {
    pub movies: Vec<Movie>,
    pub total: usize,
}

/// Normalise un label de genre TMDb en clé interne minuscule sans accents
/// "Comédie" -> "comedie", "Science-Fiction" -> "science-fiction"
pub fn normalize_genre(genre: &str) -> String {
    genre
        .to_lowercase()
        .replace('é', "e")
        .replace('è', "e")
        .replace('ê', "e")
        .replace('à', "a")
        .replace('â', "a")
        .replace('ô', "o")
        .replace('û', "u")
        .replace('î', "i")
        .replace('ï', "i")
        .replace('ç', "c")
        .replace("histoire/biopic", "histoire")
}

/// Retourne la matrice d'affinité pour un contexte humeur + météo.
/// Chaque genre reçoit un score entre 0.0 (aucune affinité) et 1.0 (affinité parfaite).
/// Les genres absents de la map ont une affinité implicite de 0.0.
pub fn get_affinities(mood: &str, condition: &str) -> HashMap<&'static str, f32> {
    let is_rain = matches!(condition, "Pluie" | "Averses");
    let is_storm = condition == "Orageux";
    let is_sun = condition == "Ensoleillé";
    let is_cloudy = matches!(condition, "Nuageux" | "Brouillard");
    let is_snow = condition == "Neige";

    let entries: &[(&str, f32)] = match mood {
        "joy" => {
            if is_sun {
                &[("comedie", 1.0), ("romance", 0.9), ("animation", 0.8), ("familial", 0.6), ("aventure", 0.5)]
            } else if is_rain {
                &[("comedie", 0.8), ("romance", 0.8), ("animation", 0.7), ("familial", 0.5)]
            } else if is_storm {
                &[("comedie", 0.7), ("animation", 0.7), ("familial", 0.6)]
            } else if is_snow {
                &[("animation", 0.9), ("familial", 0.9), ("comedie", 0.8), ("romance", 0.5)]
            } else {
                &[("comedie", 0.8), ("animation", 0.7), ("familial", 0.7), ("romance", 0.5)]
            }
        }
        "sad" => {
            if is_rain {
                &[("drame", 1.0), ("romance", 0.9), ("histoire", 0.7), ("documentaire", 0.5)]
            } else if is_storm {
                &[("drame", 1.0), ("thriller", 0.7), ("histoire", 0.5)]
            } else if is_sun {
                &[("drame", 0.7), ("histoire", 0.8), ("romance", 0.6), ("comedie", 0.4)]
            } else if is_snow {
                &[("drame", 0.8), ("romance", 0.9), ("familial", 0.6)]
            } else {
                &[("drame", 0.9), ("romance", 0.7), ("histoire", 0.6)]
            }
        }
        "angry" => {
            if is_storm {
                &[("action", 1.0), ("thriller", 1.0), ("guerre", 0.9), ("science-fiction", 0.5)]
            } else if is_rain {
                &[("action", 0.9), ("thriller", 0.9), ("guerre", 0.8)]
            } else if is_sun {
                &[("action", 0.9), ("aventure", 0.8), ("thriller", 0.7), ("science-fiction", 0.5)]
            } else if is_snow {
                &[("action", 0.8), ("thriller", 0.8), ("guerre", 0.7)]
            } else {
                &[("action", 0.9), ("thriller", 0.8), ("guerre", 0.7)]
            }
        }
        "calm" => {
            if is_sun {
                &[("documentaire", 0.9), ("animation", 0.8), ("familial", 0.8), ("romance", 0.6)]
            } else if is_cloudy {
                &[("documentaire", 1.0), ("histoire", 0.9), ("romance", 0.7), ("drame", 0.5)]
            } else if is_rain {
                &[("documentaire", 0.9), ("histoire", 0.8), ("drame", 0.7), ("romance", 0.5)]
            } else if is_snow {
                &[("documentaire", 0.9), ("animation", 0.9), ("familial", 0.8), ("romance", 0.5)]
            } else {
                &[("documentaire", 0.8), ("histoire", 0.8), ("animation", 0.7)]
            }
        }
        "tired" => {
            if is_rain {
                &[("familial", 1.0), ("animation", 1.0), ("comedie", 0.7), ("fantastique", 0.6)]
            } else if is_snow {
                &[("familial", 1.0), ("animation", 1.0), ("comedie", 0.8), ("romance", 0.5)]
            } else if is_storm {
                &[("animation", 0.9), ("familial", 0.9), ("comedie", 0.7)]
            } else if is_sun {
                &[("animation", 0.8), ("comedie", 0.8), ("romance", 0.7), ("familial", 0.6)]
            } else {
                &[("animation", 0.9), ("familial", 0.9), ("comedie", 0.7)]
            }
        }
        "fear" => {
            if is_sun {
                &[("comedie", 1.0), ("familial", 0.9), ("romance", 0.8), ("animation", 0.6)]
            } else if is_storm {
                &[("thriller", 0.9), ("horreur", 0.8), ("action", 0.5)]
            } else if is_rain {
                &[("comedie", 0.8), ("familial", 0.7), ("thriller", 0.6), ("animation", 0.5)]
            } else if is_snow {
                &[("familial", 0.9), ("animation", 0.8), ("comedie", 0.7)]
            } else {
                &[("comedie", 0.8), ("familial", 0.7), ("thriller", 0.5)]
            }
        }
        "adventure" => {
            if is_sun {
                &[("aventure", 1.0), ("fantastique", 1.0), ("science-fiction", 0.9), ("action", 0.8), ("animation", 0.4)]
            } else if is_rain {
                &[("aventure", 0.9), ("fantastique", 0.8), ("science-fiction", 0.8), ("action", 0.6)]
            } else if is_storm {
                &[("action", 1.0), ("aventure", 0.9), ("guerre", 0.8), ("science-fiction", 0.6)]
            } else if is_snow {
                &[("aventure", 1.0), ("fantastique", 0.9), ("action", 0.8), ("animation", 0.5)]
            } else {
                &[("aventure", 0.9), ("fantastique", 0.9), ("science-fiction", 0.8), ("action", 0.6)]
            }
        }
        "reflection" => {
            if is_rain {
                &[("documentaire", 1.0), ("drame", 1.0), ("science-fiction", 0.8), ("histoire", 0.7)]
            } else if is_cloudy {
                &[("documentaire", 1.0), ("histoire", 0.9), ("science-fiction", 0.8), ("drame", 0.6)]
            } else if is_sun {
                &[("histoire", 0.8), ("documentaire", 0.8), ("drame", 0.7), ("romance", 0.4)]
            } else if is_snow {
                &[("documentaire", 0.9), ("science-fiction", 0.9), ("histoire", 0.8), ("drame", 0.6)]
            } else if is_storm {
                &[("drame", 0.9), ("thriller", 0.8), ("science-fiction", 0.8), ("documentaire", 0.6)]
            } else {
                &[("documentaire", 0.9), ("histoire", 0.8), ("drame", 0.8), ("science-fiction", 0.6)]
            }
        }
        _ => &[("comedie", 0.3), ("action", 0.3), ("drame", 0.3)],
    };

    entries.iter().copied().collect()
}

/// Calcule le score d'affinité d'un film pour un contexte humeur + météo.
/// score = (rating / 10.0) × (0.4 + 0.6 × affinité_moyenne)
/// - affinité = 1.0 → score max = rating / 10
/// - affinité = 0.0 → score min = rating × 0.04 (film hors contexte mais pas invisible)
pub fn compute_affinity_score(movie: &Movie, mood: &str, condition: &str) -> f32 {
    let affinities = get_affinities(mood, condition);

    if movie.genres.is_empty() {
        return movie.rating / 10.0 * 0.4;
    }

    let total_affinity: f32 = movie
        .genres
        .iter()
        .map(|g| {
            let key = normalize_genre(g);
            affinities.get(key.as_str()).copied().unwrap_or(0.0)
        })
        .sum::<f32>();

    let avg_affinity = total_affinity / movie.genres.len() as f32;
    (movie.rating / 10.0) * (0.4 + 0.6 * avg_affinity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_genre() {
        assert_eq!(normalize_genre("Comédie"), "comedie");
        assert_eq!(normalize_genre("Science-Fiction"), "science-fiction");
        assert_eq!(normalize_genre("Histoire/Biopic"), "histoire");
        assert_eq!(normalize_genre("Fantastique"), "fantastique");
    }

    #[test]
    fn test_affinities_not_empty() {
        let a = get_affinities("joy", "Ensoleillé");
        assert!(!a.is_empty());
        assert!(a.values().all(|&v| v >= 0.0 && v <= 1.0));
    }

    #[test]
    fn test_affinity_score_in_context() {
        let movie = Movie {
            id: 1,
            title: "Test".to_string(),
            genres: vec!["Comédie".to_string()],
            rating: 8.0,
            poster_path: None,
            overview: String::new(),
            release_date: None,
        };
        // joy + soleil → comedie = 1.0 → score = 0.8 * (0.4 + 0.6) = 0.8
        let score = compute_affinity_score(&movie, "joy", "Ensoleillé");
        assert!((score - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_affinity_score_out_of_context() {
        let movie = Movie {
            id: 2,
            title: "Test".to_string(),
            genres: vec!["Horreur".to_string()],
            rating: 8.0,
            poster_path: None,
            overview: String::new(),
            release_date: None,
        };
        // joy + soleil → horreur = 0.0 → score = 0.8 * 0.4 = 0.32
        let score = compute_affinity_score(&movie, "joy", "Ensoleillé");
        assert!((score - 0.32).abs() < 0.001);
    }

    #[test]
    fn test_affinity_score_multi_genre() {
        let movie = Movie {
            id: 3,
            title: "Test".to_string(),
            genres: vec!["Action".to_string(), "Science-Fiction".to_string()],
            rating: 10.0,
            poster_path: None,
            overview: String::new(),
            release_date: None,
        };
        // adventure + soleil → action=0.8, science-fiction=0.9 → avg=0.85
        // score = 1.0 * (0.4 + 0.6 * 0.85) = 0.91
        let score = compute_affinity_score(&movie, "adventure", "Ensoleillé");
        assert!((score - 0.91).abs() < 0.001);
    }

    #[test]
    fn test_affinity_score_no_genres() {
        let movie = Movie {
            id: 4,
            title: "Test".to_string(),
            genres: vec![],
            rating: 7.0,
            poster_path: None,
            overview: String::new(),
            release_date: None,
        };
        let score = compute_affinity_score(&movie, "joy", "Ensoleillé");
        assert!((score - 0.28).abs() < 0.001); // 0.7 * 0.4
    }

    #[test]
    fn test_unknown_mood_fallback() {
        let a = get_affinities("inconnu", "Ensoleillé");
        assert!(!a.is_empty());
    }
}