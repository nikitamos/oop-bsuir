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
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::{auth::Claims, routes, state::AppState},
    clients::sea_orm::{SeaOrmRepositoryFactory, setup_db},
    models::{
        db::RepositoryFactory,
        domain::{account::Account, category::Category, transaction::Transaction, user},
        dtos::{budget::BudgetResponse, category::CategoryResponse},
        entities::{
            account as account_entity, category as category_entity,
            transaction as transaction_entity, user as user_entity,
        },
    },
};

const TEST_DB: &str = "sqlite::memory:";
const TEST_SECRET: &str = "c96e54ab4eafcdb49a7a393df998e2c0452f9e8d48f90e9d20d44f099c1bbd70";

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
    let app_state = AppState::setup(repo, Some(TEST_SECRET));

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
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(TEST_SECRET.as_bytes());

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
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("insufficient")
    );
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

// ─────────────────────────────────────────────────────────────
// Tests: Account Endpoints - create_account
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_create_account_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "create_acc@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    let response = server
        .post("/api/accounts")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": "Test Account",
            "account_type": "Cash",
            "initial_balance": "250.00"
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Test Account");
    assert_eq!(body["account_type"], "Cash");
    assert_eq!(body["balance"], "250");

    // Verify account persisted in DB
    use crate::models::entities::account as account_entity;
    use sea_orm::EntityTrait;
    let account = account_entity::Entity::find()
        .filter(account_entity::Column::Name.eq("Test Account"))
        .one(&db)
        .await
        .unwrap();
    assert!(account.is_some());
    assert_eq!(account.unwrap().balance, dec!(250.0));
}

#[rstest]
#[tokio::test]
async fn test_create_account_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server
        .post("/api/accounts")
        .json(&json!({
            "name": "Test Account",
            "account_type": "cash",
            "initial_balance": "100.00"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

// ─────────────────────────────────────────────────────────────
// Tests: Transaction Endpoints - get_transactions
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_get_transactions_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    use crate::models::domain::transaction::TransactionType;
    use crate::models::entities::transaction as transaction_entity;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};

    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "get_txn@test.com", "pw").await;
    let cat_id = seed_category(&db, "Testing", user_id).await;
    let acc_id = seed_account(&db, user_id, "Test Wallet", dec!(1000)).await;

    // Seed a transaction
    let now = Utc::now();
    let txn = transaction_entity::ActiveModel {
        account_id: Set(acc_id),
        category_id: Set(cat_id),
        amount: Set(dec!(25.50)),
        transaction_type: Set(TransactionType::Expense.into()),
        description: Set(Some("Test txn".to_string())),
        transaction_date: Set(now.naive_utc()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    txn.insert(&db).await.unwrap();

    let token = generate_test_token(user_id, 3600);
    let from = now - chrono::Duration::days(7);
    let to = now + chrono::Duration::days(7);

    let response = server
        .get(&format!(
            "/api/transactions?account_id={}&from={}&to={}",
            acc_id, from.format("%Y-%m-%dT%H:%M:%S"), to.format("%Y-%m-%dT%H:%M:%S")
        ))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let transactions: Vec<serde_json::Value> = response.json();
    assert!(!transactions.is_empty());
    assert!(transactions.iter().any(|t| t["description"] == "Test txn"));
}

#[rstest]
#[tokio::test]
async fn test_get_transactions_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server
        .get("/api/transactions?account_id=1&from=2024-01-01T00:00:00&to=2024-12-31T23:59:59")
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

// ─────────────────────────────────────────────────────────────
// Tests: Budget Endpoints
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_set_budget_success(#[future] test_server_and_db: (TestServer, DatabaseConnection)) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "budget@test.com", "pw").await;
    let cat_id = seed_category(&db, "Budget Cat", user_id).await;
    let token = generate_test_token(user_id, 3600);

    let response = server
        .post("/api/budgets")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "category_id": cat_id,
            "amount_limit": "500.00",
            "month": "2024-06-15"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: BudgetResponse = response.json();
    assert_eq!(body.category_id, cat_id);
    assert_eq!(body.amount_limit, dec!(500.0));
    assert_eq!(body.month, NaiveDate::from_ymd(2024, 06, 01));
}

#[rstest]
#[tokio::test]
async fn test_set_budget_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "budget_unauth@test.com", "pw").await;
    let cat_id = seed_category(&db, "Cat", user_id).await;

    let response = server
        .post("/api/budgets")
        .json(&json!({
            "category_id": cat_id,
            "amount_limit": "100.00",
            "month": "2024-06-01"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[rstest]
#[tokio::test]
async fn test_get_budgets_success(#[future] test_server_and_db: (TestServer, DatabaseConnection)) {
    use crate::models::entities::budget as budget_entity;
    use chrono::NaiveDate;
    use sea_orm::{ActiveModelTrait, Set};

    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "get_budgets@test.com", "pw").await;
    let cat_id = seed_category(&db, "Budget Test", user_id).await;
    let token = generate_test_token(user_id, 3600);

    // Seed a budget
    let month = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
    let budget = budget_entity::ActiveModel {
        user_id: Set(user_id),
        category_id: Set(cat_id),
        amount_limit: Set(dec!(1000.0)),
        current_spent: Set(dec!(250.0)),
        month: Set(month),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    budget.insert(&db).await.unwrap();

    let response = server
        .get("/api/budgets?month=2024-07-01")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let budgets: Vec<serde_json::Value> = response.json();
    assert!(!budgets.is_empty());
    assert!(budgets.iter().any(|b| b["category_id"] == cat_id));
}

#[rstest]
#[tokio::test]
async fn test_get_budgets_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server.get("/api/budgets?month=2024-07-01").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[rstest]
#[tokio::test]
async fn test_get_budget_status_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    use crate::models::entities::budget as budget_entity;
    use chrono::NaiveDate;
    use sea_orm::{ActiveModelTrait, Set};

    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "budget_status@test.com", "pw").await;
    let cat_id = seed_category(&db, "Status Cat", user_id).await;
    let token = generate_test_token(user_id, 3600);

    // Seed a budget and get its ID
    let month = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
    let budget = budget_entity::ActiveModel {
        user_id: Set(user_id),
        category_id: Set(cat_id),
        amount_limit: Set(dec!(750.0)),
        current_spent: Set(dec!(100.0)),
        month: Set(month),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let inserted = budget.insert(&db).await.unwrap();
    let budget_id = inserted.id;

    let response = server
        .get(&format!("/api/budgets/{}", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["id"], budget_id);
    assert_eq!(body["amount_limit"], "750");
    assert_eq!(body["current_spent"], "100");
    assert_eq!(body["limit_exceeded"], false);
}

#[rstest]
#[tokio::test]
async fn test_get_budget_status_not_found(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "budget_404@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    // Try to get a non-existent budget
    let response = server
        .get("/api/budgets/99999")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[rstest]
#[tokio::test]
async fn test_get_budget_status_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server.get("/api/budgets/1").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

// ─────────────────────────────────────────────────────────────
// Tests: Analytics Endpoints
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_get_expenses_summary_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    use crate::models::domain::transaction::TransactionType;
    use crate::models::entities::transaction as transaction_entity;
    use chrono::NaiveDateTime;
    use sea_orm::{ActiveModelTrait, Set};

    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "analytics@test.com", "pw").await;
    let cat_id = seed_category(&db, "Analytics Cat", user_id).await;
    let acc_id = seed_account(&db, user_id, "Analytics Acc", dec!(5000)).await;
    let token = generate_test_token(user_id, 3600);

    // Seed a transaction in the date range
    let txn_date =
        NaiveDateTime::parse_from_str("2024-01-15T10:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();

    let txn = transaction_entity::ActiveModel {
        account_id: Set(acc_id),
        category_id: Set(cat_id),
        amount: Set(dec!(123.45)),
        transaction_type: Set(TransactionType::Expense.into()),
        description: Set(Some("Analytics test".to_string())),
        transaction_date: Set(txn_date),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    txn.insert(&db).await.unwrap();

    let response = server
        .get("/api/analytics/expenses?from=2024-01-01&to=2024-02-01")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["total_expenses"].as_str().unwrap(), "123.45");
    assert!(body["categories"].as_array().is_some());
}

#[rstest]
#[tokio::test]
async fn test_get_expenses_summary_invalid_date_range(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "analytics_bad@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    // from >= to should return error
    let response = server
        .get("/api/analytics/expenses?from=2024-02-01&to=2024-01-01")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[rstest]
#[tokio::test]
async fn test_get_expenses_summary_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server
        .get("/api/analytics/expenses?from=2024-01-01&to=2024-02-01")
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[rstest]
#[tokio::test]
async fn test_get_monthly_summary_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "monthly@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    let response = server
        .get("/api/analytics/monthly?months=3")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let summaries: Vec<serde_json::Value> = response.json();
    // Should return up to 3 months of data (may be empty if no transactions)
    assert!(summaries.len() <= 3);
}

#[rstest]
#[tokio::test]
async fn test_get_monthly_summary_default_months(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "monthly_default@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    // Omit months param to test default value of 6
    let response = server
        .get("/api/analytics/monthly")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let summaries: Vec<serde_json::Value> = response.json();
    assert!(summaries.len() <= 6);
}

#[rstest]
#[tokio::test]
async fn test_get_monthly_summary_invalid_months_zero(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "monthly_zero@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    let response = server
        .get("/api/analytics/monthly?months=0")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[rstest]
#[tokio::test]
async fn test_get_monthly_summary_invalid_months_exceeded(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "monthly_exceeded@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    let response = server
        .get("/api/analytics/monthly?months=25")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[rstest]
#[tokio::test]
async fn test_get_monthly_summary_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server.get("/api/analytics/monthly?months=3").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

// ─────────────────────────────────────────────────────────────
// Tests: Category Endpoints - create_category
// ─────────────────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn test_create_category_success(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "create_cat@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    let response = server
        .post("/api/categories")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": "New Category",
            "category_type": "expense"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "New Category");
    assert_eq!(body["category_type"], "expense");
    assert!(body["id"].as_i64().is_some());

    // Verify category persisted
    use crate::models::entities::category as category_entity;
    use sea_orm::EntityTrait;
    let category = category_entity::Entity::find()
        .filter(category_entity::Column::Name.eq("New Category"))
        .one(&db)
        .await
        .unwrap();
    assert!(category.is_some());
}

#[rstest]
#[tokio::test]
async fn test_create_category_income_type(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, db) = test_server_and_db.await;

    let user_id = seed_user(&db, "create_cat_income@test.com", "pw").await;
    let token = generate_test_token(user_id, 3600);

    let response = server
        .post("/api/categories")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": "Income Category",
            "category_type": "income"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Income Category");
    assert_eq!(body["category_type"], "income");
}

#[rstest]
#[tokio::test]
async fn test_create_category_unauthorized(
    #[future] test_server_and_db: (TestServer, DatabaseConnection),
) {
    let (server, _) = test_server_and_db.await;

    let response = server
        .post("/api/categories")
        .json(&json!({
            "name": "Unauthorized Cat",
            "category_type": "income"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
