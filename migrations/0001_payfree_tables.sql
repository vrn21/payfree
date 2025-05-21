CREATE TABLE IF NOT EXISTS Users (
    userid UUID PRIMARY KEY,
    name TEXT NOT NULL,
    username TEXT UNIQUE NOT NULL,
    phno TEXT NOT NULL,
    address TEXT NOT NULL,
    balance DOUBLE PRECISION NOT NULL DEFAULT 0,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS Transactions (
    txn_id UUID PRIMARY KEY,
    amount DOUBLE PRECISION NOT NULL,
    from_username TEXT NOT NULL REFERENCES Users(username),
    to_username TEXT NOT NULL REFERENCES Users(username),
    time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
