use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

// Durée de validité d'un token : 7 jours.
const TOKEN_TTL_SECONDS: u64 = 7 * 24 * 3600;

fn jwt_secret() -> &'static [u8] {
    static SECRET: OnceLock<Vec<u8>> = OnceLock::new();
    SECRET
        .get_or_init(|| {
            let s = std::env::var("JWT_SECRET")
                .expect("JWT_SECRET manquante : définir une valeur longue et aléatoire dans .env");
            // Un secret trop court affaiblit gravement la signature : on refuse de démarrer.
            assert!(
                s.len() >= 32,
                "JWT_SECRET trop court (minimum 32 caractères)"
            );
            s.into_bytes()
        })
        .as_slice()
}

/// Contenu signé du token. `sub` = id utilisateur, `exp` = expiration (epoch).
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: u64,
}

/// Hache un mot de passe en clair avec Argon2id + sel aléatoire.
/// Le résultat (format PHC) contient le sel et les paramètres, à stocker tel quel.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Vérifie un mot de passe en clair contre un hash stocké.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Émet un JWT signé pour un utilisateur donné.
pub fn issue_token(user_id: i64, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: now + TOKEN_TTL_SECONDS,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret()),
    )
}

/// Utilisateur authentifié, injecté dans les handlers protégés.
/// L'extraction échoue (401) si le header Authorization: Bearer <token> est
/// absent ou invalide.
pub struct AuthUser {
    pub id: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Token manquant"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Format de token invalide"))?;

        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Token invalide ou expiré"))?;

        Ok(AuthUser {
            id: data.claims.sub,
        })
    }
}
