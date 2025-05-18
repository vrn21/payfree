use actix_web::{HttpResponse, Responder, get, post};

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hi Raghav!")
}

#[post("/auth/signup")]
pub async fn new_user() -> impl Responder {
    //post details userid, name, username, phno, address, balance,
    //create new passwd, hash it, store it
    //generate jwt

    HttpResponse::Ok().body("Hello world!")
}

#[post("/auth/login")]
pub async fn login() -> impl Responder {
    //take input username, passwd, hash passwd, check in db for the password, if matching, generate jwt token

    HttpResponse::Ok().body("Hello world!")
}

#[get("/users/{username}/profile")]
pub async fn profile() -> impl Responder {
    //fetch details userid, name, username, phno, address, balance from users table give jwt token and username
    HttpResponse::Ok().body("Hello world!")
}

#[get("/users/{username}/transactions")]
pub async fn get_transactions() -> impl Responder {
    //fetch details: txn_id, amount, from_username, to_username, time from transactions table, also where the from_username || to_username == username give jwt and username
    HttpResponse::Ok().body("Hello world!")
}

#[get("/users/{username}/balance")]
pub async fn check_balance() -> impl Responder {
    //fetch details: balance from users table where username == username give jwt and username
    HttpResponse::Ok().body("Hello world!")
}

#[post("/transactions/new")]
pub async fn new_transaction() -> impl Responder {
    //insert new_uuid, amount, from_username, to_username, time to the transactions database, give jwt and username
    HttpResponse::Ok().body("Hello world!")
}

#[get("/transactions/{id}")]
pub async fn get_transaction() -> impl Responder {
    //fetch new_uuid, amount, from_username, to_username, time from transactions table given uuid, give jwt and username
    HttpResponse::Ok().body("Hello world!")
}
