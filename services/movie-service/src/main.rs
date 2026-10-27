use anyhow::Result;
use rumqttc::{Client, MqttOptions, QoS};
use serde_json::Value;
use std::time::Duration;

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

const PAGES: u32 = 10;
const MAX_MOVIES: usize = 200;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Movie Service démarré...");
    dotenvy::dotenv().ok();

    let tmdb_key = std::env::var("TMDB_API_KEY").expect("TMDB_API_KEY manquante dans .env");

    let broker_host = std::env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
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
                    let catalog = shared::MovieCatalogLite {
                        total: count,
                        movies,
                    };
                    // Payload binaire (bincode) + DTO allégé : plus compact que JSON
                    // pour rester sous la limite de taille de paquet du broker.
                    if let Ok(bytes) = bincode::serialize(&catalog) {
                        println!(
                            " {} : {} films = {} octets ({} o/film)",
                            topic_name,
                            count,
                            bytes.len(),
                            bytes.len().checked_div(count).unwrap_or(0)
                        );
                        let topic = format!("movies/{}", topic_name);
                        let _ = mqtt_client.publish(&topic, QoS::AtLeastOnce, true, bytes);
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

fn create_mqtt_client(host: &str, port: u16) -> Client {
    let mut opts = MqttOptions::new("movie-service", host, port);
    opts.set_keep_alive(Duration::from_secs(5));
    opts.set_max_packet_size(shared::MQTT_MAX_PACKET_SIZE, shared::MQTT_MAX_PACKET_SIZE);

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

async fn fetch_movies_by_genre(api_key: &str, genre_id: u32) -> Result<Vec<shared::MovieLite>> {
    let mut all_movies: Vec<shared::MovieLite> = Vec::new();

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

        let movies: Vec<shared::MovieLite> = results
            .iter()
            .filter_map(|item| {
                // Récupérer tous les genre_ids TMDb et les convertir en noms normalisés
                let genre_ids: Vec<u64> = item["genre_ids"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
                    .unwrap_or_default();

                let genres = shared::tmdb_genre_ids_to_names(&genre_ids);

                let vote_count = item["vote_count"].as_u64().unwrap_or(0) as u32;

                Some(shared::MovieLite {
                    id: item["id"].as_u64()? as u32,
                    title: item["title"].as_str()?.to_string(),
                    genres,
                    rating: shared::rating_from_votes(
                        item["vote_average"].as_f64().unwrap_or(0.0),
                        vote_count,
                    ),
                    vote_count,
                    // On ne garde que le suffixe TMDb ; l'UI reconstruit l'URL.
                    poster: item["poster_path"].as_str().map(|p| p.to_string()),
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
