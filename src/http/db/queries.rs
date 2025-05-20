use crate::http::db::model::{Transaction, User};
use crate::http::errors::{ApiError, Result};
use chrono::TimeZone;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn new_user(pool: &PgPool, user: &User) -> Result<()> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (userid, name, username, phno, address, balance, password_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING userid, name, username, phno, address, balance, password_hash
        "#,
        user.userid,
        user.name,
        user.username,
        user.phno,
        user.address,
        user.balance,
        user.password_hash
    )
    .fetch_one(pool)
    .await
    .map(|_| ())
    .map_err(ApiError::Database)
}

pub async fn login(pool: &PgPool, username: &str) -> Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        SELECT userid, name, username, phno, address, balance, password_hash FROM users WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(ApiError::Database)
}

pub async fn fetch_profile(pool: &PgPool, username: &str) -> Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        SELECT userid, name, username, phno, address, balance, password_hash FROM users WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(ApiError::Database)
}

pub async fn fetch_transactions(pool: &PgPool, username: &str) -> Result<Vec<Transaction>> {
    sqlx::query_as!(
        Transaction,
        r#"
        SELECT txn_id, amount, from_username, to_username, time FROM transactions
        WHERE from_username = $1 OR to_username = $1
        ORDER BY time DESC
        "#,
        username
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::Database)
}

pub async fn fetch_balance(pool: &PgPool, username: &str) -> Result<Option<f64>> {
    sqlx::query!(
        r#"
        SELECT balance FROM users WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(|r| r.balance))
    .map_err(ApiError::Database)
}

pub async fn insert_transaction(pool: &PgPool, txn: &Transaction) -> Result<()> {
    let mut tx = pool.begin().await.map_err(ApiError::Database)?;

    let sender_balance = sqlx::query!(
        r#"SELECT balance FROM users WHERE username = $1"#,
        txn.from_username
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::Database)?
    .map(|r| r.balance)
    .ok_or(ApiError::UserNotFound)?;

    if sender_balance < txn.amount {
        return Err(ApiError::BalanceLow);
    }

    sqlx::query!(
        r#"UPDATE users SET balance = balance - $1 WHERE username = $2"#,
        txn.amount,
        txn.from_username
    )
    .execute(&mut *tx)
    .await
    .map_err(ApiError::Database)?;

    sqlx::query!(
        r#"UPDATE users SET balance = balance + $1 WHERE username = $2"#,
        txn.amount,
        txn.to_username
    )
    .execute(&mut *tx)
    .await
    .map_err(ApiError::Database)?;

    sqlx::query!(
        r#"
        INSERT INTO transactions (txn_id, amount, from_username, to_username, time)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        txn.txn_id,
        txn.amount,
        txn.from_username,
        txn.to_username,
        txn.time.naive_utc()
    )
    .execute(&mut *tx)
    .await
    .map_err(ApiError::Database)?;

    tx.commit().await.map_err(ApiError::Database)?;
    Ok(())
}

pub async fn fetch_transaction(pool: &PgPool, txn_id: Uuid) -> Result<Option<Transaction>> {
    sqlx::query_as!(
        Transaction,
        r#"
        SELECT txn_id, amount, from_username, to_username, time FROM transactions WHERE txn_id = $1
        "#,
        txn_id
    )
    .fetch_optional(pool)
    .await
    .map_err(ApiError::Database)
}
