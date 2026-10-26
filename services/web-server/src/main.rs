use axum::extract::Path;
use axum::{
    extract::{FromRef, State},
    http::{Method, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use dotenvy::dotenv;
use rumqttc::{AsyncClient, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use shared::{
    is_known_mood, Movie, MovieCatalogLite, RecommendationRequest, RecommendationResponse,
    MQTT_MAX_PACKET_SIZE, TOPIC_RECOMMENDATION_REQUEST, TOPIC_RECOMMENDATION_RESULT,
};
use sqlx::SqlitePool;
use std::{collections::HashMap, env, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod db;
mod handlers;
mod pending;

use auth::AuthUser;
use handlers::{auth_handlers, user_handlers};
use pending::PendingRequests;

/// Au-delà, on considère que le recommendation-service ne répondra pas.
const RECOMMENDATION_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WeatherData {
    temp: f32,
    period: String,
    condition: String,
    city: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LocationData {
    lat: f32,
    lon: f32,
}

#[derive(Debug, Deserialize)]
struct MoodPayload {
    moods: Vec<String>,
}

#[derive(Clone)]
struct AppState {
    weather: Arc<Mutex<WeatherData>>,
    movies: Arc<Mutex<HashMap<String, Vec<Movie>>>>,
    pending: Arc<PendingRequests>,
    mqtt_client: AsyncClient,
    db: SqlitePool,
}

// Permet d'extraire directement un SqlitePool depuis AppState dans les handlers
// (State<SqlitePool>), tout en gardant AppState pour les routes existantes.
impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

#[derive(Debug, Serialize)]
struct MovieDetails {
    id: u32,
    title: String,
    genres: Vec<String>,
    rating: f32,
    poster_path: Option<String>,
    overview: String,
    release_date: Option<String>,
    duration: Option<u32>, // en minutes
    director: Option<String>,
    actors: Vec<String>, // 5 premiers
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Web Server démarré...");

    // Base de données : URL depuis l'env, valeur par défaut locale sinon.
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/cinenow.db".to_string());
    let db = db::init_pool(&database_url).await?;
    println!("Base de données prête ({database_url})");

    let broker_host = env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let broker_port: u16 = env::var("MQTT_BROKER_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .unwrap_or(1883);

    let mut mqttoptions = MqttOptions::new("web-server", broker_host, broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(MQTT_MAX_PACKET_SIZE, MQTT_MAX_PACKET_SIZE);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 20);

    let weather_state = Arc::new(Mutex::new(WeatherData::default()));
    let movies_state: Arc<Mutex<HashMap<String, Vec<Movie>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let pending_state = Arc::new(PendingRequests::default());

    let weather_clone = weather_state.clone();
    let movies_clone = movies_state.clone();
    let pending_clone = pending_state.clone();

    // Spawn l'eventloop EN PREMIER
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(rumqttc::Event::Incoming(Packet::Publish(msg))) => {
                    if msg.topic == "weather/current" {
                        // JSON (petit payload)
                        if let Ok(data) = serde_json::from_slice::<WeatherData>(&msg.payload) {
                            *weather_clone.lock().await = data;
                            println!("Météo mise à jour");
                        }
                    } else if msg.topic.starts_with("movies/") {
                        // Binaire (bincode) + DTO allégé, reconstruit en Movie complet.
                        // Le poster reste un suffixe : l'UI ajoute le préfixe TMDb.
                        if let Ok(catalog) = bincode::deserialize::<MovieCatalogLite>(&msg.payload)
                        {
                            let genre = msg.topic.replace("movies/", "");
                            let movies: Vec<Movie> =
                                catalog.movies.into_iter().map(|m| m.into_movie()).collect();
                            movies_clone.lock().await.insert(genre.clone(), movies);
                            println!("Films mis à jour : {}", genre);
                        }
                    } else if msg.topic == TOPIC_RECOMMENDATION_RESULT {
                        // JSON (petit payload : liste d'id). La boucle se borne à
                        // réveiller le handler en attente ; c'est lui qui
                        // reconstruit les films, sans tenir le verrou ici.
                        if let Ok(response) =
                            serde_json::from_slice::<RecommendationResponse>(&msg.payload)
                        {
                            let count = response.movie_ids.len();
                            if pending_clone.resolve(response.request_id, response.movie_ids) {
                                println!(
                                    "Recommandations reçues (requête {}) : {} films",
                                    response.request_id, count
                                );
                            } else {
                                println!(
                                    "Réponse ignorée (requête {} inconnue ou déjà traitée)",
                                    response.request_id
                                );
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Erreur MQTT: {:?}", e),
                _ => {}
            }
        }
    });

    // ENSUITE les subscribes
    client
        .subscribe("weather/current", QoS::AtLeastOnce)
        .await?;
    client
        .subscribe(TOPIC_RECOMMENDATION_RESULT, QoS::AtLeastOnce)
        .await?;
    for topic in MOVIE_TOPICS {
        client.subscribe(*topic, QoS::AtLeastOnce).await?;
    }

    let app_state = AppState {
        weather: weather_state,
        movies: movies_state,
        pending: pending_state,
        mqtt_client: client,
        db,
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        // Routes existantes
        .route("/weather", get(get_weather))
        .route("/movies", get(get_movies))
        .route("/location", post(post_location))
        .route("/movie/:id", get(get_movie_details))
        // Recommandation : authentifiée, une réponse par requête
        .route("/recommendations", post(post_recommendations))
        // Authentification
        .route("/auth/register", post(auth_handlers::register))
        .route("/auth/login", post(auth_handlers::login))
        // Espace utilisateur (protégé par token)
        .route("/me/likes", get(user_handlers::list_likes))
        .route(
            "/me/likes/:movie_id",
            post(user_handlers::add_like).delete(user_handlers::remove_like),
        )
        .route("/me/watched", get(user_handlers::list_watched))
        .route(
            "/me/watched/:movie_id",
            post(user_handlers::add_watched).delete(user_handlers::remove_watched),
        )
        .route(
            "/me/favorite-genres",
            get(user_handlers::list_favorite_genres).post(user_handlers::add_favorite_genre),
        )
        .route(
            "/me/favorite-genres/:genre",
            delete(user_handlers::remove_favorite_genre),
        )
        .with_state(app_state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    println!("API dispo sur http://localhost:3001");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_weather(State(state): State<AppState>) -> Json<WeatherData> {
    Json(state.weather.lock().await.clone())
}

async fn get_movies(State(state): State<AppState>) -> Json<HashMap<String, Vec<Movie>>> {
    Json(state.movies.lock().await.clone())
}

async fn get_movie_details(Path(id): Path<u32>) -> Json<serde_json::Value> {
    let tmdb_key = std::env::var("TMDB_API_KEY").unwrap_or_default();

    // Fetch détails + crédits en parallèle
    let details_url = format!(
        "https://api.themoviedb.org/3/movie/{}?api_key={}&language=fr-FR",
        id, tmdb_key
    );
    let credits_url = format!(
        "https://api.themoviedb.org/3/movie/{}/credits?api_key={}&language=fr-FR",
        id, tmdb_key
    );

    let client = reqwest::Client::new();
    let (details_res, credits_res) = tokio::join!(
        client.get(&details_url).send(),
        client.get(&credits_url).send(),
    );

    let details: serde_json::Value = match details_res {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => return Json(serde_json::json!({ "error": "Erreur TMDb" })),
    };

    let credits: serde_json::Value = match credits_res {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => serde_json::json!({}),
    };

    // Extraire réalisateur
    let director = credits["crew"]
        .as_array()
        .and_then(|crew| crew.iter().find(|p| p["job"] == "Director"))
        .and_then(|d| d["name"].as_str())
        .map(|s| s.to_string());

    // Extraire acteurs (5 premiers)
    let actors: Vec<String> = credits["cast"]
        .as_array()
        .map(|cast| {
            cast.iter()
                .take(5)
                .filter_map(|a| a["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Extraire genres
    let genres: Vec<String> = details["genres"]
        .as_array()
        .map(|g| {
            g.iter()
                .filter_map(|genre| genre["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let movie = MovieDetails {
        id,
        title: details["title"].as_str().unwrap_or("").to_string(),
        genres,
        rating: details["vote_average"].as_f64().unwrap_or(0.0) as f32,
        // Suffixe uniquement : l'UI reconstruit l'URL (cohérent avec la grille).
        poster_path: details["poster_path"].as_str().map(|p| p.to_string()),
        overview: details["overview"].as_str().unwrap_or("").to_string(),
        release_date: details["release_date"].as_str().map(|s| s.to_string()),
        duration: details["runtime"].as_u64().map(|d| d as u32),
        director,
        actors,
    };

    Json(serde_json::to_value(movie).unwrap_or_default())
}

/// Demande une recommandation et attend sa réponse.
///
/// `AuthUser` n'est pas encore lu : la route est authentifiée pour qu'un compte
/// ne puisse pas recevoir le calcul d'un autre, et le profil n'entrera dans le
/// calcul qu'avec les préférences utilisateur (Partie B).
async fn post_recommendations(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(payload): Json<MoodPayload>,
) -> Result<Json<Vec<Movie>>, (StatusCode, String)> {
    if payload.moods.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Aucune humeur sélectionnée".to_string(),
        ));
    }

    if let Some(unknown) = payload.moods.iter().find(|mood| !is_known_mood(mood)) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Humeur inconnue : {unknown}"),
        ));
    }

    // Sans catalogue, la réponse ne pourrait être que vide : autant le dire tout
    // de suite plutôt que d'attendre le timeout.
    if state.movies.lock().await.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Catalogue pas encore reçu, réessaie dans un instant".to_string(),
        ));
    }

    let (request_id, receiver) = state.pending.register();
    let request = RecommendationRequest {
        request_id,
        moods: payload.moods,
    };

    let published = match serde_json::to_string(&request) {
        Ok(json) => state
            .mqtt_client
            .publish(TOPIC_RECOMMENDATION_REQUEST, QoS::AtLeastOnce, false, json)
            .await
            .is_ok(),
        Err(_) => false,
    };

    if !published {
        state.pending.forget(request_id);
        return Err((
            StatusCode::BAD_GATEWAY,
            "Demande de recommandation non transmise".to_string(),
        ));
    }

    let movie_ids = match tokio::time::timeout(RECOMMENDATION_TIMEOUT, receiver).await {
        Ok(Ok(movie_ids)) => movie_ids,
        // Sender relâché sans réponse : ne devrait pas arriver, mais on ne veut
        // pas attendre pour rien.
        Ok(Err(_)) => {
            return Err((
                StatusCode::BAD_GATEWAY,
                "Réponse de recommandation perdue".to_string(),
            ))
        }
        Err(_) => {
            state.pending.forget(request_id);
            return Err((
                StatusCode::GATEWAY_TIMEOUT,
                "Le calcul a pris trop de temps".to_string(),
            ));
        }
    };

    // Reconstituer les films complets depuis le catalogue, dans l'ordre des
    // scores. Les ids absents du catalogue sont perdus ici : l'étape /movies/by-ids
    // permettra de les compléter via TMDb.
    let movies = state.movies.lock().await;
    let catalog: HashMap<u32, &Movie> = movies.values().flatten().map(|m| (m.id, m)).collect();

    Ok(Json(
        movie_ids
            .iter()
            .filter_map(|id| catalog.get(id).map(|movie| (*movie).clone()))
            .collect(),
    ))
}

async fn post_location(
    State(state): State<AppState>,
    Json(payload): Json<LocationData>,
) -> Json<serde_json::Value> {
    println!(
        "Position reçue (Lat: {}, Lon: {})",
        payload.lat, payload.lon
    );
    if let Ok(mqtt_msg) = serde_json::to_string(&payload) {
        let _ = state
            .mqtt_client
            .publish("weather/location", QoS::AtLeastOnce, false, mqtt_msg)
            .await;
    }
    Json(serde_json::json!({ "status": "success" }))
}
