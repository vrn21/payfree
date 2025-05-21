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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use uuid::Uuid;

    async fn setup_test_db() -> PgPool {
        let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_new_user_and_login() {
        let pool = setup_test_db().await;
        let username = format!("testuser_{}", Uuid::new_v4());
        let user = User {
            userid: Uuid::new_v4(),
            name: "Test User".to_string(),
            username: username.clone(),
            phno: "1234567890".to_string(),
            address: "Test Address".to_string(),
            balance: 100.0,
            password_hash: "hash".to_string(),
        };
        // Insert user
        let res = new_user(&pool, &user).await;
        assert!(res.is_ok());

        // Login user
        let found = login(&pool, &username).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, username);
    }

    #[tokio::test]
    async fn test_fetch_profile_and_balance() {
        let pool = setup_test_db().await;
        let username = format!("testuser_{}", Uuid::new_v4());
        let user = User {
            userid: Uuid::new_v4(),
            name: "Test User".to_string(),
            username: username.clone(),
            phno: "1234567890".to_string(),
            address: "Test Address".to_string(),
            balance: 123.45,
            password_hash: "hash".to_string(),
        };
        new_user(&pool, &user).await.unwrap();

        let profile = fetch_profile(&pool, &username).await.unwrap();
        assert!(profile.is_some());
        assert_eq!(profile.as_ref().unwrap().balance, 123.45);

        let balance = fetch_balance(&pool, &username).await.unwrap();
        assert_eq!(balance, Some(123.45));
    }

    #[tokio::test]
    async fn test_insert_and_fetch_transaction() {
        let pool = setup_test_db().await;
        let user1 = User {
            userid: Uuid::new_v4(),
            name: "Sender".to_string(),
            username: format!("sender_{}", Uuid::new_v4()),
            phno: "1111111111".to_string(),
            address: "Sender Address".to_string(),
            balance: 500.0,
            password_hash: "hash".to_string(),
        };
        let user2 = User {
            userid: Uuid::new_v4(),
            name: "Receiver".to_string(),
            username: format!("receiver_{}", Uuid::new_v4()),
            phno: "2222222222".to_string(),
            address: "Receiver Address".to_string(),
            balance: 100.0,
            password_hash: "hash".to_string(),
        };
        new_user(&pool, &user1).await.unwrap();
        new_user(&pool, &user2).await.unwrap();

        let txn_id = Uuid::new_v4();
        let txn = Transaction {
            txn_id,
            amount: 50.0,
            from_username: user1.username.clone(),
            to_username: user2.username.clone(),
            time: Utc::now(),
        };
        let res = insert_transaction(&pool, &txn).await;
        assert!(res.is_ok());

        let fetched = fetch_transaction(&pool, txn_id).await.unwrap();
        assert!(fetched.is_some());
        let fetched_txn = fetched.unwrap();
        assert_eq!(fetched_txn.amount, 50.0);
        assert_eq!(fetched_txn.from_username, user1.username);
        assert_eq!(fetched_txn.to_username, user2.username);

        let txns = fetch_transactions(&pool, &user1.username).await.unwrap();
        assert!(txns.iter().any(|t| t.txn_id == txn_id));
    }

    #[tokio::test]
    async fn test_balance_low_error() {
        let pool = setup_test_db().await;
        let user1 = User {
            userid: Uuid::new_v4(),
            name: "Sender".to_string(),
            username: format!("sender2_{}", Uuid::new_v4()),
            phno: "1111111111".to_string(),
            address: "Sender Address".to_string(),
            balance: 10.0,
            password_hash: "hash".to_string(),
        };
        let user2 = User {
            userid: Uuid::new_v4(),
            name: "Receiver".to_string(),
            username: format!("receiver2_{}", Uuid::new_v4()),
            phno: "2222222222".to_string(),
            address: "Receiver Address".to_string(),
            balance: 100.0,
            password_hash: "hash".to_string(),
        };
        new_user(&pool, &user1).await.unwrap();
        new_user(&pool, &user2).await.unwrap();

        let txn = Transaction {
            txn_id: Uuid::new_v4(),
            amount: 100.0, // more than sender's balance
            from_username: user1.username.clone(),
            to_username: user2.username.clone(),
            time: Utc::now(),
        };
        let res = insert_transaction(&pool, &txn).await;
        assert!(matches!(res, Err(ApiError::BalanceLow)));
    }
}
