use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
    Argon2,
};



pub fn hash_password(plain: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default().hash_password(plain.as_bytes())?.to_string())
}

pub fn verify_password(plain: &str, hash: &str) -> bool{
    PasswordHash::new(hash)
        .map(|parsed| Argon2::default().verify_password(plain.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}

