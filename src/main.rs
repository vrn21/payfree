use actix_web::{App, HttpServer};
use anyhow::Context;
use http::routes::{
    check_balance, get_transaction, get_transactions, hello, login, new_transaction, new_user,
    profile,
};
use sqlx::postgres::PgPoolOptions;
pub mod http;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = dotenvy::var("DATABASE_URL")
        // The error from `var()` doesn't mention the environment variable.
        .context("DATABASE_URL must be set")?;

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .context("failed to connect to DATABASE_URL")?;

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(new_user)
            .service(login)
            .service(profile)
            .service(get_transactions)
            .service(check_balance)
            .service(new_transaction)
            .service(get_transaction)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/*
what to do rest:
    look how to do auth,jwt?
    then look how to properly query, what is the most efficent way to do so?
    then integrate everything together.
*/
