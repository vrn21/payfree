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
