use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i32,
    iat: i64,
    exp: i64,
    pub(crate) ver: i32,
}

pub fn encode_token(
    secret: &[u8],
    sub: i32,
    ver: i32,
    ttl_secs: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = jsonwebtoken::get_current_timestamp() as i64;
    let claims = Claims {
        sub,
        iat: now,
        exp: now + ttl_secs,
        ver,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret),
    )
}

pub fn decode_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret),
        &jsonwebtoken::Validation::default(),
    )
    .map(|d| d.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let t = encode_token(b"secret", 7, 3, 60).unwrap();
        let c = decode_token(&t, b"secret").unwrap();
        assert_eq!(c.sub, 7);
        assert_eq!(c.ver, 3);
    }

    #[test]
    fn wrong_secret_fails() {
        let t = encode_token(b"secret", 7, 0, 60).unwrap();
        assert!(decode_token(&t, b"other").is_err());
    }

    #[test]
    fn expired_fails() {
        let t = encode_token(b"secret", 7, 0, -1).unwrap();
        assert!(decode_token(&t, b"secret").is_err());
    }
}
