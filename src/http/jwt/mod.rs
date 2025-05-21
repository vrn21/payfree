pub mod extractor;

use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, TokenData, errors::Error as JwtError};
use std::time::{SystemTime, UNIX_EPOCH};
use log::{debug, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject: username or user id
    pub exp: usize,  // expiration timestamp
}

pub fn generate_jwt(sub: &str, secret: &str, expiry_seconds: u64) -> Result<String, JwtError> {
    debug!("Generating JWT for subject: {}", sub);
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + expiry_seconds;

    let claims = Claims {
        sub: sub.to_owned(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    );
    match &token {
        Ok(_) => debug!("JWT generated successfully for subject: {}", sub),
        Err(e) => error!("Failed to generate JWT for subject {}: {:?}", sub, e),
    }
    token
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<TokenData<Claims>, JwtError> {
    debug!("Decoding JWT token: {}", token);
    let result = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    );
    match &result {
        Ok(data) => debug!("JWT decoded successfully for sub: {}", data.claims.sub),
        Err(e) => error!("JWT decoding failed: {:?}", e),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_generate_and_decode_jwt() {
        let secret = "test_secret";
        let username = "testuser";
        let expiry_seconds = 3600;
        let token = generate_jwt(username, secret, expiry_seconds).expect("JWT generation failed");
        assert!(!token.is_empty());

        let decoded = decode_jwt(&token, secret).expect("JWT decoding failed");
        assert_eq!(decoded.claims.sub, username);

        // Check expiration is in the future
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert!(decoded.claims.exp as u64 > now);
    }

    #[test]
    fn test_decode_jwt_with_wrong_secret_fails() {
        let secret = "test_secret";
        let wrong_secret = "wrong_secret";
        let username = "testuser";
        let expiry_seconds = 3600;
        let token = generate_jwt(username, secret, expiry_seconds).expect("JWT generation failed");
        let result = decode_jwt(&token, wrong_secret);
        assert!(result.is_err());
    }
}
