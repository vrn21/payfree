use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use crate::http::jwt::{decode_jwt, Claims};
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
            Err(_) => return ready(Err(actix_web::error::ErrorInternalServerError("JWT secret not set"))),
        };

        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    match decode_jwt(token, &secret) {
                        Ok(token_data) => {
                            return ready(Ok(AuthenticatedUser {
                                username: token_data.claims.sub,
                            }));
                        }
                        Err(_) => {
                            return ready(Err(actix_web::error::ErrorUnauthorized("Invalid JWT token")));
                        }
                    }
                }
            }
        }
        ready(Err(actix_web::error::ErrorUnauthorized("Missing or malformed Authorization header")))
    }
}
