use crate::http::jwt::decode_jwt;
use actix_web::{Error, FromRequest, HttpRequest, dev::Payload};
use futures::future::{Ready, ready};
use log::debug;
use std::env;

pub struct AuthenticatedUser {
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let secret = match env::var("JWT_SECRET") {
            Ok(s) => s,
            Err(_) => {
                log::error!("JWT secret not set in environment");
                return ready(Err(actix_web::error::ErrorInternalServerError(
                    "JWT secret not set",
                )));
            }
        };

        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    debug!("Attempting to decode JWT for incoming request");
                    match decode_jwt(token, &secret) {
                        Ok(token_data) => {
                            debug!(
                                "JWT successfully decoded for user: {}",
                                token_data.claims.sub
                            );
                            return ready(Ok(AuthenticatedUser {
                                username: token_data.claims.sub,
                            }));
                        }
                        Err(e) => {
                            log::warn!("Invalid JWT token: {:?}", e);
                            return ready(Err(actix_web::error::ErrorUnauthorized(
                                "Invalid JWT token",
                            )));
                        }
                    }
                }
            }
        }
        debug!("Missing or malformed Authorization header for JWT authentication");
        ready(Err(actix_web::error::ErrorUnauthorized(
            "Missing or malformed Authorization header",
        )))
    }
}
