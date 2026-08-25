use anyhow::Error;
use rand::RngCore;
use rand::rngs::OsRng;
//use rand::{Rng, distributions::Alphanumeric};
use sha2::{Sha256, Digest};
use argon2::{Argon2, password_hash, PasswordHash, PasswordHasher, PasswordVerifier};
use secrecy::{SecretString, ExposeSecret};
use shared::ApiError;


const PASSWORD_LENGTH: usize = 12;  
const PASSWORD_COMPLEXITY: usize = 3;  // From 1 to 4


// Validate the strength of a password against the set values
pub async fn validate_password_strength(pw: &str) -> Result<(), ApiError> {
    if pw.len() < PASSWORD_LENGTH { 
        let msg = format!("Password must be at least {} characters", PASSWORD_LENGTH);
        return Err(ApiError { message: msg.into() });
    }

    let has_lower = pw.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = pw.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = pw.chars().any(|c| c.is_ascii_digit());
    let has_symbol = pw.chars().any(|c| !c.is_ascii_alphanumeric());

    let categories = has_lower as usize + has_upper as usize + has_digit as usize + has_symbol as usize;

    if categories < PASSWORD_COMPLEXITY {
        let msg = format!("Password must combine at least {} of the following: Lowercase, Uppercase, Digits, Symbols", PASSWORD_COMPLEXITY);
        return Err(ApiError { message: msg.into() });
    }

    let forbidden = [
        "password", "123456", "qwerty", "letmein",
        "admin", "test", "changeme"
    ];

    if forbidden.iter().any(|bad| pw.eq_ignore_ascii_case(bad)) {
        return Err(ApiError { message: "Password is too common".into() });
    }

    Ok(())
}


// Hash a password using Argon2
pub async fn hash_password(password: &SecretString) -> Result<String, Error> {
    let salt = password_hash::SaltString::generate(&mut OsRng);
    let argon = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65536, 2, 1, None)
            .map_err(|e| anyhow::anyhow!("Params failed: {}", e))?,
    );
    let hash = argon
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?;
    Ok(hash.to_string())
}


// Verify a password against a hash using Argon2
pub async fn verify_password(password: &SecretString, hash: &str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Password hash parsing failed: {}", e))?;
    let check = Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &parsed_hash).is_ok();
    Ok(check)
}


/*
// Generate a temporary password of the set length
pub async fn generate_temp_password() -> String {
    let mut rng = rand::thread_rng();

    // 12–16 characters of high‑entropy randomness
    let base: String = (0..PASSWORD_LENGTH)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();

    // Add at least one symbol to meet strength rules
    let symbols = ['!', '@', '#', '$', '%', '&', '*'];
    let symbol = symbols[rng.gen_range(0..symbols.len())];

    format!("{}{}", base, symbol)
}
*/

// Generate a random token
pub fn generate_reset_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);

    let hash = Sha256::digest(&bytes);
    hex::encode(hash)
}


// Email the reset password link to the user
//pub async fn send_reset_link(email: &str, link: &str) -> Result<bool, Error> {
    //TODO: Implement function 
//}


//-------------------------------------------------------------
// Unit tests
//-------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_short_passwords() {
        let res = validate_password_strength("abc").await;
        assert!(res.is_err(), "expected short password to be rejected");
    }

    #[tokio::test]
    async fn rejects_common_passwords() {
        let res = validate_password_strength("password").await;
        assert!(res.is_err(), "expected common password to be rejected");
    }

    #[tokio::test]
    async fn accepts_strong_password() {
        // meets length and category requirements
        let res = validate_password_strength("Abcdef1!2345").await;
        assert!(res.is_ok(), "expected strong password to be accepted");
    }
}


