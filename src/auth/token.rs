use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i32,
    iat: i64,
    exp: i64,
    pub(crate) ver: i32
}

pub fn encode_token(secret: &[u8], sub: i32, ver: i32, ttl_secs: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let now = jsonwebtoken::get_current_timestamp() as i64;
    let claims = Claims { sub, iat: now, exp: now + ttl_secs, ver };
    jsonwebtoken::encode(&jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
                         &claims,
                         &jsonwebtoken::EncodingKey::from_secret(secret))
}

pub fn decode_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret),
        &jsonwebtoken::Validation::default(),
    ).map(|d|d.claims)
}