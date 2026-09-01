use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::auth::{hash_password, issue_token, verify_password};

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
}

// Règles minimales de validation (à durcir avant prod : complexité, longueur max...).
const MIN_USERNAME_LEN: usize = 3;
const MIN_PASSWORD_LEN: usize = 8;

/// POST /auth/register — crée un compte et renvoie un token.
pub async fn register(
    State(pool): State<SqlitePool>,
    Json(creds): Json<Credentials>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    if creds.username.len() < MIN_USERNAME_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Nom d'utilisateur trop court (min {MIN_USERNAME_LEN})"),
        ));
    }
    if creds.password.len() < MIN_PASSWORD_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Mot de passe trop court (min {MIN_PASSWORD_LEN})"),
        ));
    }

    let password_hash = hash_password(&creds.password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erreur de hachage".into(),
        )
    })?;

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&creds.username)
        .bind(&password_hash)
        .execute(&pool)
        .await;

    let user_id = match result {
        Ok(r) => r.last_insert_rowid(),
        // Violation de contrainte UNIQUE = username déjà pris.
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err((StatusCode::CONFLICT, "Nom d'utilisateur déjà pris".into()));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erreur base de données".into(),
            ));
        }
    };

    let token = issue_token(user_id, &creds.username)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Erreur de token".into()))?;

    Ok(Json(AuthResponse {
        token,
        username: creds.username,
    }))
}

/// POST /auth/login — vérifie les identifiants et renvoie un token.
pub async fn login(
    State(pool): State<SqlitePool>,
    Json(creds): Json<Credentials>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let row: Option<(i64, String)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE username = ?")
            .bind(&creds.username)
            .fetch_optional(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erreur base de données".into(),
                )
            })?;

    // Message identique que l'utilisateur existe ou non : ne pas révéler
    // quels comptes existent (bonne pratique de sécurité).
    let (user_id, password_hash) =
        row.ok_or((StatusCode::UNAUTHORIZED, "Identifiants invalides".into()))?;

    if !verify_password(&creds.password, &password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "Identifiants invalides".into()));
    }

    let token = issue_token(user_id, &creds.username)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Erreur de token".into()))?;

    Ok(Json(AuthResponse {
        token,
        username: creds.username,
    }))
}
