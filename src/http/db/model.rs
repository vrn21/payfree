use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Executor;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub userid: Uuid,
    pub name: String,
    pub username: String,
    pub phno: String,
    pub address: String,
    pub balance: f64,
    pub password_hash: String,
}



#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub txn_id: Uuid,
    pub amount: f64,
    pub from_username: String,
    pub to_username: String,
    pub time: DateTime<Utc>,
}

pub async fn init_db(pool: &PgPool) -> anyhow::Result<()> {
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Users (
            userid UUID PRIMARY KEY,
            name TEXT NOT NULL,
            username TEXT UNIQUE NOT NULL,
            phno TEXT NOT NULL,
            address TEXT NOT NULL,
            balance DOUBLE PRECISION NOT NULL DEFAULT 0,
            password_hash TEXT NOT NULL
        );
        "#,
    )
    .await?;

    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Transactions (
            txn_id UUID PRIMARY KEY,
            amount DOUBLE PRECISION NOT NULL,
            from_username TEXT NOT NULL REFERENCES Users(username),
            to_username TEXT NOT NULL REFERENCES Users(username),
            time TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .await?;
    Ok(())
}
