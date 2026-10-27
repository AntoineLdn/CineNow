use chrono::Utc;
use dotenvy::dotenv;
use rumqttc::{AsyncClient, MqttOptions, Packet, QoS};
use shared::{
    RecommendationRequest, RecommendationResponse, MQTT_MAX_PACKET_SIZE,
    TOPIC_RECOMMENDATION_REQUEST, TOPIC_RECOMMENDATION_RESULT,
};
use std::{collections::HashMap, env, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use recommendation_service::{compute_recommendations, WeatherData};

// Topics entrants
const TOPIC_WEATHER: &str = "weather/current";
const MOVIE_TOPICS: &[&str] = &[
    "movies/comedie",
    "movies/romance",
    "movies/animation",
    "movies/drame",
    "movies/action",
    "movies/thriller",
    "movies/guerre",
    "movies/horreur",
    "movies/familial",
    "movies/aventure",
    "movies/fantastique",
    "movies/science-fiction",
    "movies/documentaire",
    "movies/histoire",
];

#[derive(Clone)]
struct State {
    weather: Arc<Mutex<WeatherData>>,
    movies: Arc<Mutex<HashMap<String, Vec<shared::Movie>>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Recommendation Service démarré...");

    let broker_host = env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let broker_port: u16 = env::var("MQTT_BROKER_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .unwrap_or(1883);

    let mut mqttoptions = MqttOptions::new("recommendation-service", broker_host, broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(MQTT_MAX_PACKET_SIZE, MQTT_MAX_PACKET_SIZE);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 20);

    let state = State {
        weather: Arc::new(Mutex::new(WeatherData::default())),
        movies: Arc::new(Mutex::new(HashMap::new())),
    };

    let state_clone = state.clone();
    let client_clone = client.clone();

    // Spawn l'eventloop EN PREMIER
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(rumqttc::Event::Incoming(Packet::Publish(msg))) => {
                    if msg.topic == TOPIC_WEATHER {
                        // JSON (petit payload, lisible dans le broker)
                        if let Ok(data) = serde_json::from_slice::<WeatherData>(&msg.payload) {
                            *state_clone.weather.lock().await = data;
                        }
                    } else if msg.topic.starts_with("movies/") {
                        // Binaire (bincode) + DTO allégé, reconstruit en Movie complet
                        match bincode::deserialize::<shared::MovieCatalogLite>(&msg.payload) {
                            Ok(catalog) => {
                                let genre = msg.topic.replace("movies/", "");
                                let movies: Vec<shared::Movie> =
                                    catalog.movies.into_iter().map(|m| m.into_movie()).collect();
                                state_clone.movies.lock().await.insert(genre, movies);
                            }
                            // Typiquement un movie-service d'une autre version : sans
                            // ce log, le service calculerait sur un catalogue vide.
                            Err(e) => eprintln!(
                                "Catalogue illisible sur {} : {} — movie-service est-il à jour ?",
                                msg.topic, e
                            ),
                        }
                    } else if msg.topic == TOPIC_RECOMMENDATION_REQUEST {
                        // JSON (petit payload)
                        if let Ok(request) =
                            serde_json::from_slice::<RecommendationRequest>(&msg.payload)
                        {
                            println!(
                                "Requête {} : humeurs {:?}",
                                request.request_id, request.moods
                            );

                            let weather = state_clone.weather.lock().await.clone();
                            let movies = state_clone.movies.lock().await.clone();

                            let recommendations = compute_recommendations(
                                &request.moods,
                                &weather,
                                &movies,
                                Utc::now().date_naive(),
                            );

                            println!("{} films recommandés", recommendations.len());

                            // Le score ne sort pas : l'ordre de la liste le porte.
                            let response = RecommendationResponse {
                                request_id: request.request_id,
                                movie_ids: recommendations.iter().map(|r| r.id).collect(),
                            };

                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = client_clone
                                    .publish(
                                        TOPIC_RECOMMENDATION_RESULT,
                                        QoS::AtLeastOnce,
                                        false,
                                        json,
                                    )
                                    .await;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erreur MQTT: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                _ => {}
            }
        }
    });

    // ENSUITE les subscribes
    client.subscribe(TOPIC_WEATHER, QoS::AtLeastOnce).await?;
    client
        .subscribe(TOPIC_RECOMMENDATION_REQUEST, QoS::AtLeastOnce)
        .await?;
    for topic in MOVIE_TOPICS {
        client.subscribe(*topic, QoS::AtLeastOnce).await?;
    }

    println!("Abonné aux topics, en attente...");
    tokio::signal::ctrl_c().await?;
    Ok(())
}
