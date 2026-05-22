// api/openapi.rs
use utoipa::OpenApi;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

#[derive(OpenApi)]
#[openapi(
    paths(
        api::auth::register,
        api::auth::login,
        api::accounts::create_account,
        api::accounts::get_accounts,
        api::transactions::create_transaction,
        api::transactions::get_transactions,
        api::budgets::set_budget,
        api::budgets::get_budgets,
        api::budgets::get_budget_status,
        api::analytics::get_expenses_summary,
        api::analytics::get_monthly_summary,
        api::categories::get_categories,
    ),
    components(
        schemas(
            RegisterRequest, LoginRequest, AuthResponse,
            CreateAccountRequest, AccountResponse,
            CreateTransactionRequest, TransactionResponse,
            WarningResponse<TransactionResponse>,
            SetBudgetRequest, BudgetResponse,
            ExpensesSummary, CategorySummary, MonthlySummary,
            ErrorResponse,  CategoryResponse
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "accounts", description = "Account management"),
        (name = "transactions", description = "Income and expense transactions"),
        (name = "budgets", description = "Monthly budget limits"),
        (name = "analytics", description = "Spending analytics and reports"),
        (name = "categories", description = "Transaction categories reference"),
    ),
    info(
        title = "Personal Finance Tracker API",
        version = "1.0.0",
        description = "API для отслеживания личных финансов"
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

// Монтирование Swagger UI в роутер
pub fn swagger_routes() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
// main.rs
use axum::Router;
use std::sync::Arc;

use crate::{api, application, clients, domain, shared};

#[tokio::main]
async fn main() {
    // Инициализация БД
    let db = clients::db::connect().await;
    clients::db::run_migrations(&db).await;

    // Репозитории (SeaORM реализации)
    let user_repo = Arc::new(clients::user_repo::SeaOrmUserRepo::new(db.clone()));
    let account_repo = Arc::new(clients::account_repo::SeaOrmAccountRepo::new(db.clone()));
    let category_repo = Arc::new(clients::category_repo::SeaOrmCategoryRepo::new(db.clone()));
    let transaction_repo = Arc::new(clients::transaction_repo::SeaOrmTransactionRepo::new(
        db.clone(),
    ));
    let budget_repo = Arc::new(clients::budget_repo::SeaOrmBudgetRepo::new(db.clone()));

    let auth_service = Arc::new(application::AuthService::new(
        user_repo.clone(),
        std::env::var("JWT_SECRET").unwrap_or_default(),
    ));
    let account_service = Arc::new(application::AccountService::new(account_repo));
    let transaction_service = Arc::new(application::TransactionService::new(
        transaction_repo,
        account_repo,
        budget_repo,
        category_repo.clone(),
    ));
    let budget_service = Arc::new(application::BudgetService::new(
        budget_repo,
        category_repo.clone(),
    ));
    let analytics_service = Arc::new(application::AnalyticsService::new(
        transaction_repo,
        category_repo,
    ));

    // Состояние приложения
    let state = api::state::AppState {
        auth_service,
        account_service,
        transaction_service,
        budget_service,
        analytics_service,
    };

    // Сборка роутера со Swagger
    let app = api::build_router(state).merge(api::openapi::swagger_routes());

    // Запуск сервера
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    println!("Swagger UI: http://localhost:3000/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
