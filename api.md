# API Documentation

This document provides details on the API endpoints, including the HTTP method, endpoint path, descriptions, request bodies, response formats, and example `curl` commands.

---

## Endpoints

### GET /

- **Description:** Returns a greeting message.
- **Response:** A simple greeting message ("Hi Raghav!").
- **Example `curl` command:**
  ```sh
  curl http://localhost:4040/
  ```

---

### POST /auth/signup

- **Description:** Create a new user account.
- **Request Body:** Should include:
  - `userid`: Unique identifier for the user (UUID).
  - `name`: Full name of the user (String).
  - `username`: Desired username (String).
  - `phno`: Phone number (String).
  - `address`: User address (String).
  - `balance`: Initial balance (Double).
  - `password`: Plaintext password (String) that will be hashed and stored.
- **Response:** A JSON object containing a JWT token upon successful signup.
  ```json
  {
    "token": "<JWT_TOKEN>"
  }
  ```
- **Additional Notes:** The API hashes the provided password using Argon2 and stores the hash. A JWT token is generated for the new user.
- **Example `curl` command:**
  ```sh
  curl -X POST http://localhost:4040/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "userid": "55555556-5555-5555-5555-555555555555",
    "name": "Ayush Agarwal",
    "username": "ayush2",
    "phno": "5555555555",
    "address": "Bangalore",
    "balance": 600.0,
    "password": "password5"
  }'
  ```

---

### POST /auth/login

- **Description:** Authenticate a user.
- **Request Body:** Should include:
  - `username`: The user’s username (String).
  - `password`: The user’s password (String).
- **Response:** A JSON object containing a JWT token upon successful login.
  ```json
  {
    "token": "<JWT_TOKEN>"
  }
  ```
- **Additional Notes:** The API retrieves the user by username, hashes the provided password, and verifies it against the stored hash. If valid, a JWT token is generated.
- **Example `curl` command:**
  ```sh
  curl -X POST http://localhost:4040/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "ayush2",
    "password": "password5"
  }'
  ```

---

### GET /users/{username}/profile

- **Description:** Retrieve the profile details of a user.
- **Path Parameter:**
  - `username`: Username of the user whose profile is being requested (String).
- **Response:** A JSON object containing user details.
  ```json
  {
    "userid": "55555556-5555-5555-5555-555555555555",
    "name": "Ayush Agarwal",
    "username": "ayush2",
    "phno": "5555555555",
    "address": "Bangalore",
    "balance": 600.0,
    "password_hash": "<hashed_password>"
  }
  ```
- **Additional Notes:** Requires a valid JWT token in the `Authorization` header. The token's subject (`sub` claim) must match the requested username.
- **Example `curl` command:**
  ```sh
  curl http://localhost:4040/users/ayush2/profile \
  -H "Authorization: Bearer <JWT_TOKEN>"
  ```

---

### GET /users/{username}/transactions

- **Description:** Retrieve the transaction history for a user.
- **Path Parameter:**
  - `username`: Username of the user whose transactions are being fetched (String).
- **Response:** A JSON array containing a list of transactions.
  ```json
  [
    {
      "txn_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
      "amount": 50.0,
      "from_username": "ayush2",
      "to_username": "bhargav",
      "time": "2024-05-03T10:00:00Z"
    }
  ]
  ```
- **Additional Notes:** Requires a valid JWT token in the `Authorization` header. The API fetches transactions where the user is either the sender or receiver.
- **Example `curl` command:**
  ```sh
  curl http://localhost:4040/users/ayush2/transactions \
  -H "Authorization: Bearer <JWT_TOKEN>"
  ```

---

### GET /users/{username}/balance

- **Description:** Check the account balance for a user.
- **Path Parameter:**
  - `username`: Username of the user (String).
- **Response:** A JSON number representing the current balance.
  ```json
  600.0
  ```
- **Additional Notes:** Requires a valid JWT token in the `Authorization` header. The token's subject (`sub` claim) must match the requested username.
- **Example `curl` command:**
  ```sh
  curl http://localhost:4040/users/ayush2/balance \
  -H "Authorization: Bearer <JWT_TOKEN>"
  ```

---

### POST /transactions/new

- **Description:** Create a new transaction.
- **Request Body:** Should include:
  - `txn_id`: Unique identifier for the transaction (UUID).
  - `amount`: Transaction amount (Double).
  - `from_username`: Sender's username (String).
  - `to_username`: Receiver's username (String).
  - `time`: Timestamp of the transaction (String in RFC3339 format).
- **Response:** A simple message to be sent  OK returns transaction id with an OK
  ```text
  Transaction inserted
  ```
- **Additional Notes:** Requires a valid JWT token in the `Authorization` header. The token's subject (`sub` claim) must match the `from_username` in the request body. The system verifies that the sender has sufficient balance.


  first lets create a new user:

  ```sh
  curl -X POST http://localhost:4040/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "userid": "55555558-5555-5555-5555-555555555555",
    "name": "Bhargav",
    "username": "bhargav",
    "phno": "5555555555",
    "address": "Bangalore",
    "balance": 900.0,
    "password": "password9"
  }'

 - **Example `curl` command for transacting:**
  ```sh
  curl -X POST http://localhost:4040/transactions/new \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <JWT_TOKEN>" \
  -d '{
    "txn_id": "aaaaaaab-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
    "amount": 100,
    "from_username": "ayush2",
    "to_username": "bob",
    "time": "2024-05-30T12:00:00Z"
  }'
```

---

### GET /transactions/{id}

- **Description:** Retrieve details for a specific transaction.
- **Path Parameter:**
  - `id`: Unique identifier of the transaction (UUID).
- **Response:** A JSON object containing transaction details.
  ```json
  {
    "txn_id": "aaaaaaab-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
    "amount": 50.0,
    "from_username": "ayush2",
    "to_username": "bob",
    "time": "2024-05-30T12:00:00Z"
  }
  ```
- **Additional Notes:** Requires a valid JWT token in the `Authorization` header.
- **Example `curl` command:**
  ```sh
  curl http://localhost:4040/transactions/aaaaaaab-aaaa-aaaa-aaaa-aaaaaaaaaaaa \
  -H "Authorization: Bearer <JWT_TOKEN>"
  ```
