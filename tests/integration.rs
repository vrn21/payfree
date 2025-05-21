use actix_web::{App, test};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use actix_web::web;
use sqlx::postgres::PgPoolOptions;

#[actix_rt::test]
async fn test_signup_login_profile_balance_transaction() {
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(payfree::http::routes::init_routes),
    )
    .await;

    let test_users = vec![
        (
            "11111111-1111-1111-1111-111111111111",
            "Rishabh Goel",
            "rishabh",
            "9999999999",
            "Delhi",
            1000.0,
            "password1",
        ),
        (
            "22222222-2222-2222-2222-222222222222",
            "Anurag B.",
            "anurag",
            "8888888888",
            "Mumbai",
            900.0,
            "password2",
        ),
        (
            "33333333-3333-3333-3333-333333333333",
            "Deep Doshi",
            "deep",
            "7777777777",
            "Ahmedabad",
            800.0,
            "password3",
        ),
        (
            "44444444-4444-4444-4444-444444444444",
            "Joshua D'Costa",
            "joshua",
            "6666666666",
            "Goa",
            700.0,
            "password4",
        ),
        (
            "55555555-5555-5555-5555-555555555555",
            "Ayush Agarwal",
            "ayush",
            "5555555555",
            "Bangalore",
            600.0,
            "password5",
        ),
        (
            "66666666-6666-6666-6666-666666666666",
            "Raghavendra Muppirisetty",
            "raghavendra",
            "4444444444",
            "Hyderabad",
            500.0,
            "password6",
        ),
    ];

    for (userid, name, username, phno, address, balance, password) in &test_users {
        let req = test::TestRequest::post()
            .uri("/auth/signup")
            .set_json(&json!({
                "userid": userid,
                "name": name,
                "username": username,
                "phno": phno,
                "address": address,
                "balance": balance,
                "password": password
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        if !resp.status().is_success() {
            let body = test::read_body(resp).await;
            println!("Signup failed: {:?}", body);
            panic!("Signup failed");
        }

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("token").is_some());
    }

    let mut tokens = Vec::new();
    for (_, _, username, _, _, _, password) in &test_users {
        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({
                "username": username,
                "password": password
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        let token = body.get("token").unwrap().as_str().unwrap().to_string();
        tokens.push((username.to_string(), token));
    }

    for (i, (_, _, username, _, _, balance, _)) in test_users.iter().enumerate() {
        let token = &tokens[i].1;
        let req = test::TestRequest::get()
            .uri(&format!("/users/{}/profile", username))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["username"], *username);

        let req = test::TestRequest::get()
            .uri(&format!("/users/{}/balance", username))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body, json!(balance));
    }

    let txn_id = Uuid::new_v4();
    let req = test::TestRequest::post()
        .uri("/transactions/new")
        .insert_header(("Authorization", format!("Bearer {}", tokens[0].1)))
        .set_json(&json!({
            "txn_id": txn_id,
            "amount": 100,
            "from_username": "rishabh",
            "to_username": "anurag",
            "time": Utc::now().to_rfc3339()
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri("/users/rishabh/transactions")
        .insert_header(("Authorization", format!("Bearer {}", tokens[0].1)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().len() > 0);

    let req = test::TestRequest::get()
        .uri("/users/anurag/transactions")
        .insert_header(("Authorization", format!("Bearer {}", tokens[1].1)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().len() > 0);

    let req = test::TestRequest::get()
        .uri(&format!("/transactions/{}", txn_id))
        .insert_header(("Authorization", format!("Bearer {}", tokens[0].1)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["txn_id"], json!(txn_id.to_string()));
}
