use crate::http::db::model;
use crate::http::db::queries;
use crate::http::errors::ApiError;
use crate::http::jwt::extractor::AuthenticatedUser;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hi Raghav!")
}

use crate::http::db::model::User;
use crate::http::jwt::{Claims, generate_jwt};
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
    let req = req.into_inner();
    let password_hash = passwd::hash(req.password)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
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
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret".to_string());
    let token =
        generate_jwt(&req.username, &secret, 3600).map_err(|_| ApiError::InternalServerError)?;
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
    let req = req.into_inner();
    let user = queries::login(&pool, &req.username)
        .await?
        .ok_or(ApiError::UserNotFound)?;
    let valid = passwd::verify(req.password, user.password_hash)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    if !valid {
        return Err(ApiError::InvalidCredentials);
    }
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret".to_string());
    let token =
        generate_jwt(&user.username, &secret, 3600).map_err(|_| ApiError::InternalServerError)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token })))
}
#[get("/users/{username}/profile")]
pub async fn profile(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let username = path.into_inner();
    if username != user.username {
        return Err(ApiError::Unauthorized);
    }
    let user = queries::fetch_profile(&pool, &username)
        .await?
        .ok_or(ApiError::UserNotFound)?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/users/{username}/transactions")]
pub async fn get_transactions(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let username = path.into_inner();
    if username != user.username {
        return Err(ApiError::Unauthorized);
    }
    let txns = queries::fetch_transactions(&pool, &username).await?;
    Ok(HttpResponse::Ok().json(txns))
}

#[get("/users/{username}/balance")]
pub async fn check_balance(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let username = path.into_inner();
    if username != user.username {
        return Err(ApiError::Unauthorized);
    }
    let balance = queries::fetch_balance(&pool, &username)
        .await?
        .ok_or(ApiError::UserNotFound)?;
    Ok(HttpResponse::Ok().json(balance))
}

#[post("/transactions/new")]
pub async fn new_transaction(
    pool: web::Data<PgPool>,
    txn: web::Json<model::Transaction>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    // Only allow if the sender is the authenticated user
    if txn.from_username != user.username {
        return Err(ApiError::Unauthorized);
    }
    match queries::insert_transaction(&pool, &txn.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok().body("Transaction inserted")),
        Err(ApiError::BalanceLow) => Ok(HttpResponse::BadRequest().body("Insufficient balance")),
        Err(e) => Err(e),
    }
}

#[get("/transactions/{id}")]
pub async fn get_transaction(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let txn_id = path.into_inner();
    let txn = queries::fetch_transaction(&pool, txn_id)
        .await?
        .ok_or(ApiError::UserNotFound)?;
    Ok(HttpResponse::Ok().json(txn))
}
