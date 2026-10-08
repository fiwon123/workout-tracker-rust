use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
    Argon2,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub iat: i64,
    pub exp: i64,
}

pub fn hash_password(plain: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default().hash_password(plain.as_bytes())?.to_string())
}

pub fn verify_password(plain: &str, hash: &str) -> bool{
    PasswordHash::new(hash)
        .map(|parsed| Argon2::default().verify_password(plain.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}

pub fn decode_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret),
        &jsonwebtoken::Validation::default(),
    ).map(|d|d.claims)
}