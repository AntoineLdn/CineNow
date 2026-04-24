use anyhow::Result;
use rumqttc::{MqttOptions, Client, QoS};
use std::time::Duration;
use serde_json::Value;

const GENRES: &[(&str, u32)] = &[
    ("comedie", 35),
    ("romance", 10749),
    ("animation", 16),
    ("drame", 18),
    ("action", 28),
    ("thriller", 53),
    ("guerre", 10752),
    ("horreur", 27),
    ("familial", 10751),
    ("aventure", 12),
    ("fantastique", 14),
    ("science-fiction", 878),
    ("documentaire", 99),
    ("histoire", 36),
];

// IDs TMDb → nom normalisé (liste complète des genres TMDb)
const TMDB_GENRE_MAP: &[(u32, &str)] = &[
    (28, "action"),
    (12, "aventure"),
    (16, "animation"),
    (35, "comedie"),
    (80, "crime"),
    (99, "documentaire"),
    (18, "drame"),
    (10751, "familial"),
    (14, "fantastique"),
    (36, "histoire"),
    (27, "horreur"),
    (10402, "musique"),
    (9648, "mystere"),
    (10749, "romance"),
    (878, "science-fiction"),
    (10770, "tele-film"),
    (53, "thriller"),
    (10752, "guerre"),
    (37, "western"),
];

const PAGES: u32 = 5;
const MAX_MOVIES: usize = 30;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Movie Service démarré...");
    dotenvy::dotenv().ok();

    let tmdb_key = std::env::var("TMDB_API_KEY")
        .expect("TMDB_API_KEY manquante dans .env");

    let broker_host = std::env::var("MQTT_BROKER_HOST")
        .unwrap_or_else(|_| "localhost".to_string());
    let broker_port: u16 = std::env::var("MQTT_BROKER_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .unwrap_or(1883);

    let mqtt_client = create_mqtt_client(&broker_host, broker_port);
    println!("Connecté à MQTT");

    loop {
        println!("Récupération des films par genre...");

        for (topic_name, genre_id) in GENRES {
            match fetch_movies_by_genre(&tmdb_key, *genre_id).await {
                Ok(movies) => {
                    let count = movies.len();
                    let catalog = shared::MovieCatalog { total: count, movies };
                    println!(" {} : {} films", topic_name, count);
                    if let Ok(json) = serde_json::to_string(&catalog) {
                        let topic = format!("movies/{}", topic_name);
                        let _ = mqtt_client.publish(&topic, QoS::AtLeastOnce, true, json.as_bytes());
                    }
                }
                Err(e) => eprintln!(" Erreur {} : {}", topic_name, e),
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        println!("Tous les genres publiés sur MQTT !");
        println!("Prochain fetch dans 24h...");
        tokio::time::sleep(Duration::from_secs(86400)).await;
    }
}

fn tmdb_genre_ids_to_names(ids: &[u64]) -> Vec<String> {
    ids.iter()
        .filter_map(|id| {
            TMDB_GENRE_MAP
                .iter()
                .find(|(tmdb_id, _)| *tmdb_id == *id as u32)
                .map(|(_, name)| name.to_string())
        })
        .collect()
}

fn create_mqtt_client(host: &str, port: u16) -> Client {
    let mut opts = MqttOptions::new("movie-service", host, port);
    opts.set_keep_alive(Duration::from_secs(5));

    let (client, mut connection) = Client::new(opts, 10);

    std::thread::spawn(move || {
        for notification in connection.iter() {
            if let Err(e) = notification {
                eprintln!("MQTT Error: {}", e);
            }
        }
    });

    std::thread::sleep(Duration::from_millis(500));
    client
}

async fn fetch_movies_by_genre(api_key: &str, genre_id: u32) -> Result<Vec<shared::Movie>> {
    let mut all_movies: Vec<shared::Movie> = Vec::new();

    for page in 1..=PAGES {
        let url = format!(
            "https://api.themoviedb.org/3/discover/movie?api_key={}&language=fr-FR&with_genres={}&sort_by=popularity.desc&page={}",
            api_key, genre_id, page
        );

        let response = reqwest::get(&url).await?;
        let json: Value = response.json().await?;

        let results = match json["results"].as_array() {
            Some(r) => r,
            None => break,
        };

        let movies: Vec<shared::Movie> = results
            .iter()
            .filter_map(|item| {
                let overview = item["overview"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(100)
                    .collect::<String>();

                // Récupérer tous les genre_ids TMDb et les convertir en noms normalisés
                let genre_ids: Vec<u64> = item["genre_ids"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
                    .unwrap_or_default();

                let genres = tmdb_genre_ids_to_names(&genre_ids);

                Some(shared::Movie {
                    id: item["id"].as_u64()? as u32,
                    title: item["title"].as_str()?.to_string(),
                    genres,
                    rating: item["vote_average"].as_f64()? as f32,
                    poster_path: item["poster_path"]
                        .as_str()
                        .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                    overview,
                    release_date: item["release_date"].as_str().map(|d| d.to_string()),
                })
            })
            .collect();

        all_movies.extend(movies);
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    all_movies.dedup_by_key(|m| m.id);
    all_movies.truncate(MAX_MOVIES);
    Ok(all_movies)
}