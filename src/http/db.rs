use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub userid: Uuid,
    pub name: String,
    pub username: String,
    pub phno: String,
    pub address: String,
    pub balance: f64,
    pub password_hash: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Transaction {
    pub txn_id: Uuid,
    pub amount: f64,
    pub from_username: String,
    pub to_username: String,
    pub time: DateTime<Utc>,
}

pub async fn new_user(pool: &PgPool, user: &User) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO Users (userid, name, username, phno, address, balance, password_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user.userid,
        user.name,
        user.username,
        user.phno,
        user.address,
        user.balance,
        user.password_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn login(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM Users WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn fetch_profile(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM Users WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn fetch_transactions(pool: &PgPool, username: &str) -> Result<Vec<Transaction>> {
    let txns = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM Transactions
        WHERE from_username = $1 OR to_username = $1
        ORDER BY time DESC
        "#,
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    Ok(txns)
}

pub async fn fetch_balance(pool: &PgPool, username: &str) -> Result<Option<f64>> {
    let rec = sqlx::query!(
        r#"
        SELECT balance FROM Users WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await?;
    Ok(rec.map(|r| r.balance))
}

pub async fn insert_transaction(pool: &PgPool, txn: &Transaction) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO Transactions (txn_id, amount, from_username, to_username, time)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        txn.txn_id,
        txn.amount,
        txn.from_username,
        txn.to_username,
        txn.time
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_transaction(pool: &PgPool, txn_id: Uuid) -> Result<Option<Transaction>> {
    let txn = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM Transactions WHERE txn_id = $1
        "#,
    )
    .bind(txn_id)
    .fetch_optional(pool)
    .await?;
    Ok(txn)
}
