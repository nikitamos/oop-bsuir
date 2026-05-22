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
        pub category_service: Arc<CategoryController>,
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
    shared::error::ApplicationError,
};
use axum::{
    Extension, Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use chrono::{Datelike, NaiveDate};
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
    let account = state
        .account_service
        .create_account(
            user_id,
            request.name,
            request.account_type.into(),
            request.initial_balance,
        )
        .await?;
    Ok((StatusCode::OK, Json(account.into())))
}

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
    Ok(Json(
        state
            .account_service
            .get_accounts(user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
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
    // Validate that account belongs to the user?
    let transaction = state
        .transaction_service
        .create_transaction(
            request.account_id,
            request.category_id,
            request.amount,
            request.transaction_type.into(),
            request.description,
        )
        .await?;
    // Check whether the budget has been exceeded
    Ok((
        StatusCode::CREATED,
        Json(TransactionResponse::from_transaction(transaction, false)),
    ))
}

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
    // Validate that accounts belong to the user
    let res = state
        .transaction_service
        .get_transactions(params.account_id, params.from, params.to)
        .await?
        .into_iter()
        .map(|tr| TransactionResponse::from_transaction(tr, false))
        .collect();
    Ok(Json(res))
}

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
    let budget = state
        .budget_service
        .set_budget(
            user_id,
            request.category_id,
            request.amount_limit,
            request.month.with_day(1).unwrap(),
        )
        .await
        .map(From::from)?;

    Ok((StatusCode::CREATED, Json(budget)))
}

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
    let budgets = state
        .budget_service
        .get_budgets(user_id, month.month.with_day(1).unwrap())
        .await?;

    let responses: Vec<BudgetResponse> = budgets.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

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
    // Validate that budget belongs to the user
    let budget = state.budget_service.get_budget_status(budget_id).await?;

    Ok(Json(BudgetResponse::from(budget)))
}

// api/analytics.rs

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
    if params.from >= params.to {
        return Err(AppError(ApplicationError::BadRequest(
            "'from' date must be earlier than 'to' date".to_string(),
        )));
    }

    let summary = state
        .analytics_service
        .get_expenses_summary(user_id, params.from, params.to)
        .await?;

    Ok(Json(summary))
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
    let months_count = months.months.unwrap_or(6);

    if months_count == 0 {
        return Err(AppError(ApplicationError::BadRequest(
            "'months' must be greater than 0".to_string(),
        )));
    }
    if months_count > 24 {
        return Err(AppError(ApplicationError::BadRequest(
            "'months' cannot exceed 24".to_string(),
        )));
    }

    let summaries = state
        .analytics_service
        .get_monthly_summary(user_id, months_count)
        .await?;

    Ok(Json(summaries))
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
    let categories = state.category_service.get_all_categories().await?;

    let responses: Vec<CategoryResponse> = categories.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

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
