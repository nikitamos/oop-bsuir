pub mod state {
    use crate::controllers::prelude::*;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct AppState {
        pub auth_service: Arc<AuthController>,
        pub account_service: Arc<AccountController>,
        pub transaction_service: Arc<TransactionController>,
        pub budget_service: Arc<BudgetController>,
        pub analytics_service: Arc<AnalyticsController>,
    }
}
pub mod error {
    use crate::{models::dtos::ErrorResponse, shared::error::ApplicationError};
    use axum::{
        Json,
        http::StatusCode,
        response::{IntoResponse, Response},
    };

    pub struct AppError(pub ApplicationError);

    impl From<ApplicationError> for AppError {
        fn from(err: ApplicationError) -> Self {
            AppError(err)
        }
    }

    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            let (status, message) = match &self.0 {
                ApplicationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
                ApplicationError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
                ApplicationError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
                ApplicationError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
                ApplicationError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            };

            (
                status,
                Json(ErrorResponse {
                    error: status.canonical_reason().unwrap_or("Unknown").to_string(),
                    message,
                }),
            )
                .into_response()
        }
    }
}

use crate::{
    api::{error::AppError, state::AppState},
    models::dtos::{
        ErrorResponse,
        account::{AccountResponse, CreateAccountRequest},
        analytics::{AnalyticsRequest, ExpensesSummary, MonthlySummary},
        auth::{AuthResponse, LoginRequest, RegisterRequest},
        budget::{BudgetResponse, SetBudgetRequest},
        category::CategoryResponse,
        queries::{MonthCountQuery, MonthQuery},
        transaction::{CreateTransactionRequest, GetTransactionsRequest, TransactionResponse},
    },
};
use axum::{
    Extension, Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use utoipa::OpenApi;

pub struct AuthenticatedUser {
    pub user_id: i64,
    pub email: String,
}
impl<S> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async {
            Ok(Self {
                user_id: 1,
                email: "xxx@example.com".to_string(),
            })
        }
    }
}

// api/auth.rs
/// Регистрация нового пользователя
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    todo!()
}

/// Аутентификация пользователя
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    todo!()
}

// api/accounts.rs
/// Создание нового счёта
#[utoipa::path(
    post,
    path = "/api/accounts",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Account created", body = AccountResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "accounts",
    security(("jwt" = []))
)]
pub async fn create_account(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Json(request): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), AppError> {
    state.account_service.create_account(user_id, request.name, request.account_type);
    todo!()
}

/// Получение всех счетов пользователя
#[utoipa::path(
    get,
    path = "/api/accounts",
    responses(
        (status = 200, description = "List of accounts", body = Vec<AccountResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "accounts",
    security(("jwt" = []))
)]
pub async fn get_accounts(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    todo!()
}

// api/transactions.rs
/// Создание новой транзакции
#[utoipa::path(
    post,
    path = "/api/transactions",
    request_body = CreateTransactionRequest,
    responses(
        (status = 201, description = "Transaction created", body = TransactionResponse),
        (status = 400, description = "Bad request (insufficient funds, invalid category, etc.)", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Account not found", body = ErrorResponse)
    ),
    tag = "transactions",
    security(("jwt" = []))
)]
pub async fn create_transaction(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), AppError> {
    todo!()
}

/// Получение списка транзакций по счёту за период
#[utoipa::path(
    get,
    path = "/api/transactions",
    params(
        ("account_id" = i64, Query, description = "ID счёта"),
        ("from" = String, Query, description = "Дата начала периода (ISO 8601)"),
        ("to" = String, Query, description = "Дата конца периода (ISO 8601)")
    ),
    responses(
        (status = 200, description = "List of transactions", body = Vec<TransactionResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "transactions",
    security(("jwt" = []))
)]
pub async fn get_transactions(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Query(params): Query<GetTransactionsRequest>,
) -> Result<Json<Vec<TransactionResponse>>, AppError> {
    todo!()
}

// api/budgets.rs
/// Установка месячного лимита на категорию
#[utoipa::path(
    post,
    path = "/api/budgets",
    request_body = SetBudgetRequest,
    responses(
        (status = 201, description = "Budget set", body = BudgetResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "budgets",
    security(("jwt" = []))
)]
pub async fn set_budget(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Json(request): Json<SetBudgetRequest>,
) -> Result<(StatusCode, Json<BudgetResponse>), AppError> {
    todo!()
}

/// Получение всех бюджетов пользователя за указанный месяц
#[utoipa::path(
    get,
    path = "/api/budgets",
    params(
        ("month" = String, Query, description = "Месяц в формате YYYY-MM-DD (первое число)")
    ),
    responses(
        (status = 200, description = "List of budgets", body = Vec<BudgetResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "budgets",
    security(("jwt" = []))
)]
pub async fn get_budgets(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Query(month): Query<MonthQuery>,
) -> Result<Json<Vec<BudgetResponse>>, AppError> {
    todo!()
}

/// Получение детальной информации по конкретному бюджету
#[utoipa::path(
    get,
    path = "/api/budgets/{budget_id}",
    params(
        ("budget_id" = i64, Path, description = "ID бюджета")
    ),
    responses(
        (status = 200, description = "Budget details", body = BudgetResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Budget not found", body = ErrorResponse)
    ),
    tag = "budgets",
    security(("jwt" = []))
)]
pub async fn get_budget_status(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Path(budget_id): Path<i64>,
) -> Result<Json<BudgetResponse>, AppError> {
    todo!()
}

// api/analytics.rs
/// Сводка расходов за период в разрезе категорий
#[utoipa::path(
    get,
    path = "/api/analytics/expenses",
    params(
        ("from" = String, Query, description = "Дата начала периода (ISO 8601)"),
        ("to" = String, Query, description = "Дата конца периода (ISO 8601)")
    ),
    responses(
        (status = 200, description = "Expenses summary", body = ExpensesSummary),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "analytics",
    security(("jwt" = []))
)]
pub async fn get_expenses_summary(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Query(params): Query<AnalyticsRequest>,
) -> Result<Json<ExpensesSummary>, AppError> {
    todo!()
}

/// Сравнение доходов и расходов по месяцам
#[utoipa::path(
    get,
    path = "/api/analytics/monthly",
    params(
        ("months" = Option<u32>, Query, description = "Количество месяцев (по умолчанию 6)")
    ),
    responses(
        (status = 200, description = "Monthly income vs expenses", body = Vec<MonthlySummary>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "analytics",
    security(("jwt" = []))
)]
pub async fn get_monthly_summary(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Query(months): Query<MonthCountQuery>,
) -> Result<Json<Vec<MonthlySummary>>, AppError> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/api/categories",
    responses(
        (status = 200, description = "List of categories", body = Vec<CategoryResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "categories",
    security(("jwt" = []))
)]
pub async fn get_categories(
    State(state): State<AppState>,
    AuthenticatedUser { .. }: AuthenticatedUser,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    todo!()
}

// Сборка роутера
pub fn build_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login));

    let protected_routes = Router::new()
        .route("/api/accounts", get(get_accounts).post(create_account))
        .route(
            "/api/transactions",
            get(get_transactions).post(create_transaction),
        )
        .route("/api/budgets", get(get_budgets).post(set_budget))
        .route("/api/budgets/{budget_id}", get(get_budget_status))
        .route("/api/analytics/expenses", get(get_expenses_summary))
        .route("/api/analytics/monthly", get(get_monthly_summary))
        .route("/api/categories", get(get_categories));

    public_routes.merge(protected_routes).with_state(state)
}
