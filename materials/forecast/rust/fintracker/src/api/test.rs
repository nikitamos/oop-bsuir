use axum::{Router, http::StatusCode};
use axum_test::TestServer;
use chrono::{NaiveDate, Utc};
use jsonwebtoken::{Header, encode};
use rstest::{fixture, rstest};
use rust_decimal::{Decimal, dec};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::{auth::Claims, routes, state::AppState},
    clients::sea_orm::{SeaOrmRepositoryFactory, setup_db},
    models::{
        db::RepositoryFactory,
        domain::{account::Account, category::Category, transaction::Transaction, user},
        entities::{
            account as account_entity, category as category_entity,
            transaction as transaction_entity, user as user_entity,
        },
    },
};

const TEST_DB: &str = "sqlite::memory:";
const TEST_SECRET: &[u8] = b"dd"; // Must match AppState::setup

// ─────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────

#[fixture]
async fn test_server_and_db() -> (TestServer, DatabaseConnection) {
    // Initialize tracing once per test run (ignore errors if already initialized)
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("tower_http=debug,{}=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(fmt::layer().event_format(fmt::format().without_time()))
        .try_init();

    // 1. Create and setup database
    let db = Database::connect(TEST_DB).await.unwrap();
    setup_db(&db).await.unwrap();

    // 2. Create repository factory and app state
    let repo = SeaOrmRepositoryFactory::new(db.clone());
    let app_state = AppState::setup(repo);

    // 3. Build router and test server
    let router = routes(app_state).layer(TraceLayer::new_for_http());
    let server = TestServer::new(router);

    // 4. Return both server and db connection (db is Arc'd, so cloning is cheap)
    (server, db)
}

// ─────────────────────────────────────────────────────────────
// Test Helpers
// ─────────────────────────────────────────────────────────────

/// Generate a valid JWT token for a test user
fn generate_test_token(user_id: i64, expires_in_secs: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + expires_in_secs as usize;

    let claims = Claims { user_id, exp };
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(TEST_SECRET);

    encode(&Header::default(), &claims, &encoding_key).unwrap()
}

/// Seed a test user and return their ID
async fn seed_user(db: &DatabaseConnection, email: &str, password: &str) -> i64 {
    use crate::models::domain;
    let domain_user = domain::user::User::new_with_raw_password(email, password).unwrap();
    let user = user_entity::ActiveModel {
        email: Set(email.to_string()),
        password_hash: Set(domain_user.password_hash),
        ..Default::default()
    };
    let inserted = user.insert(db).await.unwrap();
    inserted.id
}

/// Seed a test category (expense type)
async fn seed_category(db: &DatabaseConnection, name: &str, user_id: i64) -> i64 {
    use crate::models::domain::category::CategoryType;

    let cat = category_entity::ActiveModel {
        name: Set(name.to_string()),
        category_type: Set(CategoryType::Expense.into()),
        user_id: Set(user_id),
        ..Default::default()
    };
    let inserted = cat.insert(db).await.unwrap();
    inserted.id
}

/// Seed a test account for a user
async fn seed_account(db: &DatabaseConnection, user_id: i64, name: &str, balance: Decimal) -> i64 {
    use crate::models::domain::account::AccountType;

    let account = account_entity::ActiveModel {
        user_id: Set(user_id),
        name: Set(name.to_string()),
        account_type: Set(AccountType::Cash.into()),
        balance: Set(balance),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let inserted = account.insert(db).await.unwrap();
    inserted.id
}

// ─────────────────────────────────────────────────────────────
// Tests: Public Endpoints
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_register_success(#[future] test_server_and_db: (TestServer, DatabaseConnection)) {
    let (server, db) = test_server_and_db.await;

    let response = server
        .post("/api/auth/register")
        .json(&json!({
            "email": "newuser@example.com",
            "password": "SecurePass123!"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: serde_json::Value = response.json();
    assert!(body["token"].as_str().is_some());
    assert_eq!(body["email"], "newuser@example.com");

    let user = user_entity::Entity::find()
        .filter(user_entity::Column::Email.eq("newuser@example.com"))
        .one(&db)
        .await
        .unwrap();
    assert!(user.is_some());
}

#[rstest]
#[tokio::test]
async fn test_login_success(#[future] test_server_and_db: (TestServer, DatabaseConnection)) {
    let (server, db) = test_server_and_db.await;

    // Seed a known user
    let user_id = seed_user(&db, "login@test.com", "hashed_pw").await;

    let response = server
        .post("/api/auth/login")
        .json(&json!({
            "email": "login@test.com",
            "password": "hashed_pw"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["token"].as_str().is_some());
}

#[rstest]
#[tokio::test]
async fn test_login_invalid_credentials(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server
        .post("/api/auth/login")
        .json(&json!({
            "email": "noexist@test.com",
            "password": "wrong"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

// ─────────────────────────────────────────────────────────────
// Tests: Protected Endpoints (with JWT)
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_get_accounts_requires_auth(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server.get("/api/accounts").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[rstest]
#[tokio::test]
async fn test_get_accounts_success(#[future] test_server_and_db: (TestServer, DatabaseConnection)) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "accounts@test.com", "pw").await;
    let _acc1 = seed_account(&db, user_id, "Checking", Decimal::from(100)).await;
    let _acc2 = seed_account(&db, user_id, "Savings", Decimal::from(500)).await;

    let token = generate_test_token(user_id, 3600);

    let response = server
        .get("/api/accounts")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let accounts: Vec<serde_json::Value> = response.json();
    assert_eq!(accounts.len(), 2);
    assert!(accounts.iter().any(|a| a["name"] == "Checking"));
}

#[rstest]
#[tokio::test]
async fn test_create_transaction_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "txn@test.com", "pw").await;
    let cat_id = seed_category(&db, "Groceries", user_id).await;
    let acc_id = seed_account(&db, user_id, "Wallet", Decimal::from(1000)).await;

    let token = generate_test_token(user_id, 3600);

    let response = server
        .post("/api/transactions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "account_id": acc_id,
            "category_id": cat_id,
            "amount": "42.50",
            "transaction_type": "expense",
            "description": "Test purchase"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: serde_json::Value = response.json();
    assert_eq!(body["amount"], "42.5");
    assert_eq!(body["description"], "Test purchase");

    // Verify transaction persisted and balance updated
    let txn = transaction_entity::Entity::find()
        .filter(transaction_entity::Column::Description.eq("Test purchase"))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(txn.amount, dec!(42.5));

    let account = account_entity::Entity::find_by_id(acc_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account.balance, dec!(957.5)); // 1000 - 42.5
}

#[rstest]
#[tokio::test]
async fn test_get_categories_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "cat@test.com", "pw").await;
    let _cat1 = seed_category(&db, "Food", user_id).await;
    let _cat2 = seed_category(&db, "Transport", user_id).await;

    let token = generate_test_token(user_id, 3600);

    let response = server
        .get("/api/categories")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let categories: Vec<serde_json::Value> = response.json();
    assert!(categories.iter().any(|c| c["name"] == "Food"));
}

// ─────────────────────────────────────────────────────────────
// Tests: Error Cases & Validation
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_create_transaction_insufficient_funds(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "broke@test.com", "pw").await;
    let cat_id = seed_category(&db, "Luxury", user_id).await;
    let acc_id = seed_account(&db, user_id, "Empty", Decimal::from(10)).await;

    let token = generate_test_token(user_id, 3600);

    let response = server
        .post("/api/transactions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "account_id": acc_id,
            "category_id": cat_id,
            "amount": "100.00",
            "transaction_type": "expense",
            "description": "Overspend"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = response.json();
    assert!(body["message"].as_str().unwrap().contains("insufficient"));
}

#[rstest]
#[tokio::test]
async fn test_invalid_jwt_token(#[future] test_server_and_db: (TestServer, DatabaseConnection)) {
    let (server, _) = test_server_and_db.await;

    let response = server
        .get("/api/accounts")
        .add_header("Authorization", "Bearer invalid.token.here")
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[rstest]
#[tokio::test]
async fn test_missing_auth_header(#[future] test_server_and_db: (TestServer, DatabaseConnection)) {
    let (server, _) = test_server_and_db.await;

    let response = server.get("/api/accounts").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
