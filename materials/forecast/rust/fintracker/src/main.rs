use fintracker::clients::sea_orm::SeaOrmRepositoryFactory;
use sea_orm::Database;
// api/openapi.rs
use utoipa::OpenApi;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

use fintracker::{api, clients};

#[derive(OpenApi)]
#[openapi(
    paths(
        api::register,
        api::login,
        api::create_account,
        api::get_accounts,
        api::create_transaction,
        api::get_transactions,
        api::set_budget,
        api::get_budgets,
        api::get_budget_status,
        api::get_expenses_summary,
        api::get_monthly_summary,
        api::get_categories,
    ),
// components(
//     schemas(
//         RegisterRequest, LoginRequest, AuthResponse,
//         CreateAccountRequest, AccountResponse,
//         CreateTransactionRequest, TransactionResponse,
//         WarningResponse<TransactionResponse>,
//         SetBudgetRequest, BudgetResponse,
//         ExpensesSummary, CategorySummary, MonthlySummary,
//         ErrorResponse,  CategoryResponse
//     )
// ),
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

use axum::Router;
use std::sync::Arc;

// use crate::models::{api, application, clients, domain, shared};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite://db.sqlite3?mode=rwc").await?;
    let repo_factory = SeaOrmRepositoryFactory::new(db);

    let state = api::state::AppState::setup(repo_factory);

    let doc = <ApiDoc as OpenApi>::openapi();
    let app = api::build_router(state)
        .merge(SwaggerUi::new("/swagger/index.html").url("/api-docs/openapi.json", doc))
        .with_state(());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    println!("Swagger UI: http://localhost:3000/swagger-ui");
    axum::serve(listener, app).await?;
    Ok(())
}
