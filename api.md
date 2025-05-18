# API Documentation

This document provides details on the API endpoints including the HTTP method, endpoint path, descriptions, required parameters, and additional notes regarding authentication and functionality.

---

## Endpoints

### GET /

- **Description:** Returns a greeting message.
- **Response:** A simple greeting message ("Hi Raghav!").

---

### POST /auth/signup

- **Description:** Create a new user account.
- **Request Body:** Should include:
  - `userid`: Unique identifier for the user.
  - `name`: Full name of the user.
  - `username`: Desired username.
  - `phno`: Phone number.
  - `address`: User address.
  - `balance`: Initial balance.
  - `password`: Plaintext password that will be hashed and stored.
- **Response:** A successful message to be sent  OK
- **Additional Notes:** After creating a new account, a JWT token should be generated.

---

### POST /auth/login

- **Description:** Authenticate a user.
- **Request Body:** Should include:
  - `username`: The user’s username.
  - `password`: The user’s password.
- **Response:** A successful message to be sent  OK jwt token can be returned too
- **Additional Notes:** The API hashes the provided password and verifies it against the stored hash. If valid, a JWT token is generated.

---

### GET /users/{username}/profile

- **Description:** Retrieve the profile details of a user.
- **Path Parameter:**
  - `username`: Username of the user whose profile is being requested.
- **Response:** User details including:
  - `userid`
  - `name`
  - `username`
  - `phno`
  - `address`
  - `balance`
- **Additional Notes:** Requires a valid JWT token for authentication and authorization.

---

### GET /users/{username}/transactions

- **Description:** Retrieve the transaction history for a user.
- **Path Parameter:**
  - `username`: Username of the user whose transactions are being fetched.
- **Response:** List of transactions containing:
  - `txn_id`
  - `amount`
  - `from_username`
  - `to_username`
  - `time` (timestamp)
- **Additional Notes:** Fetches transactions where the user is either the sender or receiver. Requires JWT authentication.

---

### GET /users/{username}/balance

- **Description:** Check the account balance for a user.
- **Path Parameter:**
  - `username`: Username of the user.
- **Response:** The current balance of the user.
- **Additional Notes:** Requires a valid JWT token for access.

---

### POST /transactions/new

- **Description:** Create a new transaction.
- **Request Body:** Should include:
  - `new_uuid`: Unique identifier for the transaction.
  - `amount`: Transaction amount.
  - `from_username`: Sender's username.
  - `to_username`: Receiver's username.
  - `time`: Timestamp of the transaction.
- **Response:** returns transaction id with an OK
- **Additional Notes:** Requires JWT token authentication.

---

### GET /transactions/{id}

- **Description:** Retrieve details for a specific transaction.
- **Path Parameter:**
  - `id`: Unique identifier of the transaction.
- **Response:** Transaction details including:
  - `txn_id`
  - `amount`
  - `from_username`
  - `to_username`
  - `time` (timestamp)
- **Additional Notes:** Requires JWT authentication.
