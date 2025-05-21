use crate::http::db::model::{Transaction, User};
use crate::http::errors::{ApiError, Result};
use log::debug;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn new_user(pool: &PgPool, user: &User) -> Result<()> {
    debug!("Inserting new user: {:?}", user.username);
    let result = sqlx::query(
        r#"
        INSERT INTO users (userid, name, username, phno, address, balance, password_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(&user.userid)
    .bind(&user.name)
    .bind(&user.username)
    .bind(&user.phno)
    .bind(&user.address)
    .bind(user.balance)
    .bind(&user.password_hash)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(ApiError::Database);
    debug!("Insert user result: {:?}", result);
    result
}

pub async fn login(pool: &PgPool, username: &str) -> Result<Option<User>> {
    debug!("Fetching user for login: {:?}", username);
    let rec = sqlx::query(
        r#"
        SELECT userid, name, username, phno, address, balance, password_hash FROM users WHERE username = $1
        "#
    )
    .bind(username)
    .fetch_optional(pool)
    .await;

    match rec {
        Ok(row) => match row {
            Some(row) => {
                debug!("User found for login: {:?}", username);
                let user = User {
                    userid: row.get("userid"),
                    name: row.get("name"),
                    username: row.get("username"),
                    phno: row.get("phno"),
                    address: row.get("address"),
                    balance: row.get("balance"),
                    password_hash: row.get("password_hash"),
                };
                Ok(Some(user))
            }
            None => {
                debug!("No user found for login: {:?}", username);
                Ok(None)
            }
        },
        Err(err) => {
            debug!("DB error during login for user {:?}: {:?}", username, err);
            Err(ApiError::Database(err))
        }
    }
}

pub async fn fetch_profile(pool: &PgPool, username: &str) -> Result<Option<User>> {
    debug!("Fetching profile for user: {:?}", username);
    let rec = sqlx::query(
        r#"
        SELECT userid, name, username, phno, address, balance, password_hash FROM users WHERE username = $1
        "#
    )
    .bind(username)
    .fetch_optional(pool)
    .await;

    match rec {
        Ok(row) => match row {
            Some(row) => {
                debug!("Profile found for user: {:?}", username);
                let user = User {
                    userid: row.get("userid"),
                    name: row.get("name"),
                    username: row.get("username"),
                    phno: row.get("phno"),
                    address: row.get("address"),
                    balance: row.get("balance"),
                    password_hash: row.get("password_hash"),
                };
                Ok(Some(user))
            }
            None => {
                debug!("No profile found for user: {:?}", username);
                Ok(None)
            }
        },
        Err(err) => {
            debug!(
                "DB error during profile fetch for user {:?}: {:?}",
                username, err
            );
            Err(ApiError::Database(err))
        }
    }
}

pub async fn fetch_transactions(pool: &PgPool, username: &str) -> Result<Vec<Transaction>> {
    debug!("Fetching transactions for user: {:?}", username);
    let rec = sqlx::query(
        r#"
        SELECT txn_id, amount, from_username, to_username, time FROM transactions
        WHERE from_username = $1 OR to_username = $1
        ORDER BY time DESC
        "#,
    )
    .bind(username)
    .fetch_all(pool)
    .await;
    match rec {
        Ok(rows) => {
            debug!(
                "Fetched {} transactions for user: {:?}",
                rows.len(),
                username
            );
            let transactions = rows
                .into_iter()
                .map(|row| Transaction {
                    txn_id: row.get("txn_id"),
                    amount: row.get("amount"),
                    from_username: row.get("from_username"),
                    to_username: row.get("to_username"),
                    time: row.get("time"),
                })
                .collect();
            Ok(transactions)
        }
        Err(e) => {
            debug!(
                "DB error during fetch_transactions for user {:?}: {:?}",
                username, e
            );
            Err(ApiError::Database(e))
        }
    }
}

pub async fn fetch_balance(pool: &PgPool, username: &str) -> Result<Option<f64>> {
    debug!("Fetching balance for user: {:?}", username);
    let rec = sqlx::query(
        r#"
        SELECT balance FROM users WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::Database)?;

    let balance = rec.and_then(|row| row.try_get("balance").ok());
    debug!("Fetched balance for user {:?}: {:?}", username, balance);
    Ok(balance)
}

pub async fn insert_transaction(pool: &PgPool, txn: &Transaction) -> Result<()> {
    debug!("Inserting transaction: {:?}", txn);
    let mut tx = pool.begin().await.map_err(ApiError::Database)?;

    let sender_balance = sqlx::query(r#"SELECT balance FROM users WHERE username = $1"#)
        .bind(&txn.from_username)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ApiError::Database)?;
    let sender_balance: f64 = match sender_balance {
        Some(balance) => balance.get("balance"),
        None => return Err(ApiError::UserNotFound),
    };
    debug!(
        "Sender balance for {}: {}",
        txn.from_username, sender_balance
    );

    if sender_balance < txn.amount {
        debug!("Insufficient balance for transaction: {:?}", txn);
        return Err(ApiError::BalanceLow);
    }

    sqlx::query(r#"UPDATE users SET balance = balance - $1 WHERE username = $2"#)
        .bind(txn.amount)
        .bind(&txn.from_username)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::Database)?;

    sqlx::query(r#"UPDATE users SET balance = balance + $1 WHERE username = $2"#)
        .bind(txn.amount)
        .bind(&txn.to_username)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::Database)?;

    let insert_result = sqlx::query(
        r#"
        INSERT INTO transactions (txn_id, amount, from_username, to_username, time)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(&txn.txn_id)
    .bind(txn.amount)
    .bind(&txn.from_username)
    .bind(&txn.to_username)
    .bind(txn.time)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::Database);

    debug!("Insert transaction result: {:?}", insert_result);

    tx.commit().await.map_err(ApiError::Database)?;
    Ok(())
}

pub async fn fetch_transaction(pool: &PgPool, txn_id: Uuid) -> Result<Option<Transaction>> {
    debug!("Fetching transaction by txn_id: {:?}", txn_id);
    let rec = sqlx::query(
        r#"
        SELECT txn_id, amount, from_username, to_username, time FROM transactions WHERE txn_id = $1
        "#,
    )
    .bind(txn_id)
    .fetch_optional(pool)
    .await;

    match rec {
        Ok(row) => match row {
            Some(row) => {
                let txn_id = row.try_get("txn_id")?;
                let amount = row.try_get("amount")?;
                let from_username = row.try_get("from_username")?;
                let to_username = row.try_get("to_username")?;
                let time = row.try_get("time")?;

                debug!("Transaction found: {:?}", txn_id);

                Ok(Some(Transaction {
                    txn_id,
                    amount,
                    from_username,
                    to_username,
                    time,
                }))
            }
            None => {
                debug!("No transaction found for txn_id: {:?}", txn_id);
                Ok(None)
            }
        },
        Err(e) => {
            debug!(
                "DB error during fetch_transaction for txn_id {:?}: {:?}",
                txn_id, e
            );
            Err(ApiError::Database(e))
        }
    }
}
