use crate::http::db::model;
use crate::http::db::queries;
use crate::http::errors::ApiError;
use crate::http::jwt::extractor::AuthenticatedUser;
use actix_web::{HttpResponse, Responder, get, post, web};
use log::{debug, error, warn};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[get("/")]
pub async fn hello() -> impl Responder {
    debug!("Received request: GET /");
    HttpResponse::Ok().body("Hi Raghav!")
}

use crate::http::db::model::User;
use crate::http::jwt::generate_jwt;
use crate::http::passwd;

use std::env;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub userid: Uuid,
    pub name: String,
    pub username: String,
    pub phno: String,
    pub address: String,
    pub balance: f64,
    pub password: String,
}

#[post("/auth/signup")]
pub async fn new_user(
    pool: web::Data<PgPool>,
    req: web::Json<SignupRequest>,
) -> Result<HttpResponse, ApiError> {
    debug!("POST /auth/signup called with username: {}", req.username);
    let req = req.into_inner();
    let password_hash = passwd::hash(req.password).await.map_err(|_| {
        error!("Password hashing failed for signup");
        ApiError::InternalServerError
    })?;
    let user = User {
        userid: req.userid,
        name: req.name,
        username: req.username.clone(),
        phno: req.phno,
        address: req.address,
        balance: req.balance,
        password_hash,
    };
    queries::new_user(&pool, &user).await?;
    debug!("User created: {}", user.username);
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret".to_string());
    let token = generate_jwt(&req.username, &secret, 3600).map_err(|_| {
        error!("JWT generation failed for signup");
        ApiError::InternalServerError
    })?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token })))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[post("/auth/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    debug!("POST /auth/login called for username: {}", req.username);
    let req = req.into_inner();
    let user = queries::login(&pool, &req.username).await?.ok_or_else(|| {
        warn!("Login failed: user not found: {}", req.username);
        ApiError::UserNotFound
    })?;
    let valid = passwd::verify(req.password, user.password_hash)
        .await
        .map_err(|_| {
            error!("Password verification failed for login");
            ApiError::InternalServerError
        })?;
    if !valid {
        warn!(
            "Login failed: invalid credentials for username: {}",
            user.username
        );
        return Err(ApiError::InvalidCredentials);
    }
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret".to_string());
    let token = generate_jwt(&user.username, &secret, 3600).map_err(|_| {
        error!("JWT generation failed for login");
        ApiError::InternalServerError
    })?;
    debug!("Login successful for username: {}", user.username);
    Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token })))
}
#[get("/users/{username}/profile")]
pub async fn profile(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    debug!("GET /users/{}/profile called by {}", path, user.username);
    let username = path.into_inner();
    if username != user.username {
        warn!(
            "Unauthorized profile access attempt: {} tried to access {}",
            user.username, username
        );
        return Err(ApiError::Unauthorized);
    }
    let user = queries::fetch_profile(&pool, &username)
        .await?
        .ok_or_else(|| {
            warn!("Profile not found for username: {}", username);
            ApiError::UserNotFound
        })?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/users/{username}/transactions")]
pub async fn get_transactions(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    debug!("Received request: GET /users/{}/transactions", path);
    let username = path.into_inner();
    if username != user.username {
        warn!(
            "Unauthorized transactions access attempt: {} as {}",
            user.username, username
        );
        return Err(ApiError::Unauthorized);
    }
    let txns = queries::fetch_transactions(&pool, &username).await?;
    debug!("Transactions fetched for username: {}", username);
    Ok(HttpResponse::Ok().json(txns))
}

#[get("/users/{username}/balance")]
pub async fn check_balance(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    debug!("GET /users/{}/balance called by {}", path, user.username);
    let username = path.into_inner();
    if username != user.username {
        warn!(
            "Unauthorized balance access attempt: {} as {}",
            user.username, username
        );
        return Err(ApiError::Unauthorized);
    }
    let balance = queries::fetch_balance(&pool, &username)
        .await?
        .ok_or_else(|| {
            warn!("Balance not found for username: {}", username);
            ApiError::UserNotFound
        })?;
    debug!("Balance fetched for username: {}", username);
    Ok(HttpResponse::Ok().json(balance))
}

#[post("/transactions/new")]
pub async fn new_transaction(
    pool: web::Data<PgPool>,
    txn: web::Json<model::Transaction>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    debug!("POST /transactions/new called by {}", user.username);
    if txn.from_username != user.username {
        warn!(
            "Unauthorized transaction attempt: {} tried to send from {}",
            user.username, txn.from_username
        );
        return Err(ApiError::Unauthorized);
    }
    match queries::insert_transaction(&pool, &txn.into_inner()).await {
        Ok(_) => {
            debug!("Transaction inserted by {}", user.username);
            Ok(HttpResponse::Ok().body("Transaction inserted"))
        }
        Err(ApiError::BalanceLow) => {
            warn!(
                "Transaction failed: insufficient balance for {}",
                user.username
            );
            Ok(HttpResponse::BadRequest().body("Insufficient balance"))
        }
        Err(e) => Err(e),
    }
}

#[get("/transactions/{id}")]
pub async fn get_transaction(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    debug!("GET /transactions/{} called", path);
    let txn_id = path.into_inner();
    let txn = queries::fetch_transaction(&pool, txn_id)
        .await?
        .ok_or_else(|| {
            warn!("Transaction not found for txn_id: {}", txn_id);
            ApiError::UserNotFound
        })?;
    Ok(HttpResponse::Ok().json(txn))
}

pub fn init_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(hello)
        .service(new_user)
        .service(login)
        .service(profile)
        .service(get_transactions)
        .service(check_balance)
        .service(new_transaction)
        .service(get_transaction);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use std::env;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn test_hello_route() {
        let app = test::init_service(App::new().service(hello)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert_eq!(body, "Hi Raghav!");
    }

    // NOTE: The following tests are illustrative and may require a test database and proper setup.
    // They are designed to show how to structure route handler tests.

    #[actix_rt::test]
    async fn test_signup_and_login_route() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(new_user)
                .service(login)
        ).await;

        let username = format!("testuser_{}", Uuid::new_v4());
        let signup_req = test::TestRequest::post()
            .uri("/auth/signup")
            .set_json(&json!({
                "userid": Uuid::new_v4(),
                "name": "Test User",
                "username": username,
                "phno": "1234567890",
                "address": "Test Address",
                "balance": 100.0,
                "password": "testpassword"
            }))
            .to_request();
        let signup_resp = test::call_service(&app, signup_req).await;
        assert!(signup_resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(signup_resp).await;
        assert!(body.get("token").is_some());

        let login_req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({
                "username": username,
                "password": "testpassword"
            }))
            .to_request();
        let login_resp = test::call_service(&app, login_req).await;
        assert!(login_resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(login_resp).await;
        assert!(body.get("token").is_some());
    }
}
