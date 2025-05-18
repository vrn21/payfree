use sqlx::PgPool;

pub async fn new_user() {}

pub async fn login() {}

pub async fn fetch_profile(pool: &PgPool, username: String) {
    let rec = sqlx::query!(
        r#"
            SELECT * FROM Users WHERE username = $1;
        "#,
        username
    );
}

pub async fn fetch_transactions(pool: &PgPool, username: String) {
    let rec = sqlx::query!(
        r#"
            SELECT * FROM Transactions WHERE Transactions.user_to =$1 OR Transactions.user_from = $2;
        "#,
        username,
        username
    );
}

pub async fn fetch_balance(pool: &PgPool, username: String) {
    let rec = sqlx::query!(
        r#"
            SELECT balance FROM Users WHERE username = $1
        "#,
        username
    );
}

pub async fn insert_transaction(pool: &PgPool, txn: Transaction) {
    let rec = sqlx::query!(
        r#"
            INSERT INTO Transactions VALUES($1,$2,$3,$4,$5)
        "#,
        txn.
    );
}

pub async fn fetch_transaction(pool: &PgPool, id: String) {
    let rec = sqlx::query!(
        r#"
            SELECT * FROM Transaction WHERE id = $1
        "#,
        id
    );
}
