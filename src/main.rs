use actix_web::{App, HttpServer, web};
use anyhow::Context;
use http::routes::{
    check_balance, get_transaction, get_transactions, hello, login, new_transaction, new_user,
    profile,
};
use log::info;
use sqlx::postgres::PgPoolOptions;
pub mod http;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let database_url = dotenvy::var("DATABASE_URL")
        // The error from `var()` doesn't mention the environment variable.
        .context("DATABASE_URL must be set")
        .unwrap();

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .context("failed to connect to DATABASE_URL")
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    // Initialize database tables
    http::db::model::init_db(&db)
        .await
        .context("failed to initialize database tables")
        .unwrap();

    info!("Starting server at http://127.0.0.1:4040");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(hello)
            .service(new_user)
            .service(login)
            .service(profile)
            .service(get_transactions)
            .service(check_balance)
            .service(new_transaction)
            .service(get_transaction)
    })
    .bind(("127.0.0.1", 4040))?
    .run()
    .await
}

/*
what to do rest:
    look how to do auth,jwt?
    then look how to properly query, what is the most efficent way to do so?
    then integrate everything together.
*/
