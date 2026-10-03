use axum::{extract::Path, extract::State, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::auth::AuthUser;

#[derive(Deserialize)]
pub struct GenrePayload {
    pub genre: String,
}

type ApiError = (StatusCode, String);

fn db_err(_: sqlx::Error) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Erreur base de données".into(),
    )
}

// ---------------------------------------------------------------- Likes

/// GET /me/likes — liste des movie_id likés par l'utilisateur.
pub async fn list_likes(
    user: AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<i64>>, ApiError> {
    let rows: Vec<(i64,)> = sqlx::query_as("SELECT movie_id FROM liked_movies WHERE user_id = ?")
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .map_err(db_err)?;
    Ok(Json(rows.into_iter().map(|(id,)| id).collect()))
}

/// POST /me/likes/:movie_id — ajoute un like (idempotent).
pub async fn add_like(
    user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(movie_id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("INSERT OR IGNORE INTO liked_movies (user_id, movie_id) VALUES (?, ?)")
        .bind(user.id)
        .bind(movie_id)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /me/likes/:movie_id — retire un like.
pub async fn remove_like(
    user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(movie_id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM liked_movies WHERE user_id = ? AND movie_id = ?")
        .bind(user.id)
        .bind(movie_id)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------- Watched

/// GET /me/watched — liste des movie_id vus.
pub async fn list_watched(
    user: AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<i64>>, ApiError> {
    let rows: Vec<(i64,)> = sqlx::query_as("SELECT movie_id FROM watched_movies WHERE user_id = ?")
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .map_err(db_err)?;
    Ok(Json(rows.into_iter().map(|(id,)| id).collect()))
}

/// POST /me/watched/:movie_id — marque un film comme vu (idempotent).
pub async fn add_watched(
    user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(movie_id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("INSERT OR IGNORE INTO watched_movies (user_id, movie_id) VALUES (?, ?)")
        .bind(user.id)
        .bind(movie_id)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /me/watched/:movie_id — retire un film des vus.
pub async fn remove_watched(
    user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(movie_id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM watched_movies WHERE user_id = ? AND movie_id = ?")
        .bind(user.id)
        .bind(movie_id)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------- Favorite genres

/// GET /me/favorite-genres — liste des genres favoris.
pub async fn list_favorite_genres(
    user: AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<String>>, ApiError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT genre FROM favorite_genres WHERE user_id = ?")
            .bind(user.id)
            .fetch_all(&pool)
            .await
            .map_err(db_err)?;
    Ok(Json(rows.into_iter().map(|(g,)| g).collect()))
}

/// POST /me/favorite-genres — ajoute un genre favori (idempotent).
pub async fn add_favorite_genre(
    user: AuthUser,
    State(pool): State<SqlitePool>,
    Json(payload): Json<GenrePayload>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("INSERT OR IGNORE INTO favorite_genres (user_id, genre) VALUES (?, ?)")
        .bind(user.id)
        .bind(payload.genre.to_lowercase())
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /me/favorite-genres/:genre — retire un genre favori.
pub async fn remove_favorite_genre(
    user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(genre): Path<String>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM favorite_genres WHERE user_id = ? AND genre = ?")
        .bind(user.id)
        .bind(genre.to_lowercase())
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}
