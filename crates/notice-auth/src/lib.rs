use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Password Hashing ───

/// Hash a plaintext password with Argon2id.
pub fn hash_password(password: &str) -> Result<String, notice_core::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| notice_core::Error::Auth(e.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored hash.
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
    pub exp: usize,  // expiration (unix timestamp)
    pub iat: usize,  // issued at
}

/// Create a JWT for an authenticated user. Valid for 24 hours.
pub fn create_token(user_id: &Uuid, secret: &str) -> Result<String, notice_core::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
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
    .map_err(|e| notice_core::Error::Auth(e.to_string()))?;
    Ok(data.claims)
}
