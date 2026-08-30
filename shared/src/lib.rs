use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

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

// Matrice et familles météo chargées depuis des fichiers de données,
// embarqués dans le binaire à la compilation.
const AFFINITIES_TOML: &str = include_str!("data/affinities.toml");
const WEATHER_FAMILIES_TOML: &str = include_str!("data/weather_families.toml");

// mood -> (famille météo ou "default") -> genre -> score
type AffinityMatrix = HashMap<String, HashMap<String, HashMap<String, f32>>>;
// condition brute -> famille météo
type WeatherFamilies = HashMap<String, String>;

fn matrix() -> &'static AffinityMatrix {
    static MATRIX: OnceLock<AffinityMatrix> = OnceLock::new();
    MATRIX.get_or_init(|| toml::from_str(AFFINITIES_TOML).expect("affinities.toml invalide"))
}

fn families() -> &'static WeatherFamilies {
    static FAMILIES: OnceLock<WeatherFamilies> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        toml::from_str(WEATHER_FAMILIES_TOML).expect("weather_families.toml invalide")
    })
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
///
/// Résolution : on déduit la famille météo de la condition, puis on prend
/// la section [mood.<famille>] si elle existe, sinon [mood.default].
/// Une humeur inconnue retombe sur [unknown.default].
pub fn get_affinities(mood: &str, condition: &str) -> HashMap<String, f32> {
    let matrix = matrix();

    let mood_table = matrix
        .get(mood)
        .or_else(|| matrix.get("unknown"))
        .expect("section [unknown.default] manquante dans affinities.toml");

    let family = families().get(condition).map(String::as_str);

    let genres = family
        .and_then(|f| mood_table.get(f))
        .or_else(|| mood_table.get("default"))
        .expect("section default manquante pour cette humeur");

    genres.clone()
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
            affinities.get(&key).copied().unwrap_or(0.0)
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
    fn test_data_files_parse() {
        // Force le chargement : panique si un des deux TOML est invalide.
        assert!(!matrix().is_empty());
        assert!(!families().is_empty());
    }

    #[test]
    fn test_affinities_not_empty() {
        let a = get_affinities("joy", "Ensoleillé");
        assert!(!a.is_empty());
        assert!(a.values().all(|&v| v >= 0.0 && v <= 1.0));
    }

    #[test]
    fn test_family_resolution() {
        // Averses et Pluie partagent la famille "rain" -> mêmes affinités.
        assert_eq!(
            get_affinities("sad", "Pluie"),
            get_affinities("sad", "Averses")
        );
    }

    #[test]
    fn test_default_fallback_on_unmapped_family() {
        // "reflection" n'a pas d'override pour la famille "snow"? Il en a un.
        // "joy" n'a pas de section "cloudy" -> doit retomber sur default.
        assert_eq!(
            get_affinities("joy", "Nuageux"),
            get_affinities("joy", "Vent inconnu")
        );
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
