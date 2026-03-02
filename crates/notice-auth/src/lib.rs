use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Password Hashing ───

pub fn hash_password(password: &str) -> Result<String, notice_core::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| notice_core::Error::Auth(e.to_string()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, notice_core::Error> {
    let parsed = PasswordHash::new(hash).map_err(|e| notice_core::Error::Auth(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// ─── JWT Tokens ───

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user ID
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

/// Create a JWT for an authenticated user. Valid for 24 hours.
pub fn create_token(
    user_id: &Uuid,
    username: &str,
    secret: &str,
) -> Result<String, notice_core::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(24)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| notice_core::Error::Auth(e.to_string()))
}

/// Verify and decode a JWT. Returns the claims if valid.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, notice_core::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| notice_core::Error::Auth(format!("Invalid token: {}", e)))?;
    Ok(data.claims)
}

/// Extract the user_id UUID from claims.
pub fn user_id_from_claims(claims: &Claims) -> Result<Uuid, notice_core::Error> {
    Uuid::parse_str(&claims.sub)
        .map_err(|e| notice_core::Error::Auth(format!("Invalid user ID in token: {}", e)))
}
