# Payfree

Payfree is a secure, JWT-authenticated backend for a simple payment and transaction management system. It allows users to sign up, log in, view their profile and balance, and perform transactions with other users. The backend is built with Rust, Actix-Web, SQLx, and PostgreSQL.

---

## 🚀 Features

- User signup and login with Argon2 password hashing
- JWT-based authentication and authorization
- View user profile, balance, and transaction history
- Create and fetch transactions
- PostgreSQL-backed persistent storage
- Modular, testable, and production-ready codebase

---

## 🛠️ Technologies Used

- **Rust** (2024 edition)
- **Actix-Web** (web framework)
- **SQLx** (async Postgres ORM)
- **PostgreSQL** (database)
- **Argon2** (password hashing)
- **jsonwebtoken** (JWT handling)
- **dotenvy** (env management)
- **env_logger/log** (logging)
- **serde** (serialization)
- **uuid** (unique IDs)
- **chrono** (timestamps)

---

## 🏗️ Architecture

```
+-------------------+        +-------------------+        +-------------------+
|    HTTP Client    | <----> |   Actix-Web App   | <----> |   PostgreSQL DB   |
+-------------------+        +-------------------+        +-------------------+
        |                           |                              |
        |      [Routes/Handlers]    |                              |
        |-------------------------> |                              |
        |                           |   [SQLx Queries/Models]      |
        |                           |----------------------------> |
        |                           |                              |
        | <------------------------ | <--------------------------- |
        |      [JSON Response]      |      [Query Results]         |
```

---

## 🌳 API Tree

```
/
├── GET /
├── /auth/
|    ├── POST /auth/signup
|    ├── POST /auth/login
├── /users/{username}/
|    ├── GET /users/{username}/profile
|    ├── GET /users/{username}/transactions
|    ├── GET /users/{username}/balance
├── /transaction/
    ├── POST /transactions/new
    └── GET /transactions/{id}
```

---

## 🗄️ Database Table Diagram

```
+-------------------+         +----------------------+
|      Users        |         |     Transactions     |
+-------------------+         +----------------------+
| userid (UUID, PK) |<------. | txn_id (UUID, PK)    |
| name (TEXT)       |       | | amount (DOUBLE)      |
| username (TEXT, UQ)|      | | from_username (TEXT) |
| phno (TEXT)       |       | | to_username (TEXT)   |
| address (TEXT)    |       | | time (TIMESTAMPTZ)   |
| balance (DOUBLE)  |       | +----------------------+
| password_hash (TEXT)|     |
+-------------------+       |
                            |
  from_username, to_username|
        (FK to Users) ------'
```

---

## 🧑‍💻 Getting Started

1. **Clone the repo**
2. **Set up PostgreSQL** and create a database
3. **Configure environment variables** (`DATABASE_URL`, `JWT_SECRET`)
4. **Run migrations**
   ```
   sqlx migrate run
   ```
5. **Build and run the server**
   ```
   cargo run
   ```
6. **Run tests**
   ```
   cargo test
   ```

---


---
