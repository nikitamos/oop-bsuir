use axum::extract::FromRef;

use crate::{api::auth::JwtState, controllers::prelude::*, repositories::RepositoryFactory};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthController>,
    pub account_service: Arc<AccountController>,
    pub transaction_service: Arc<TransactionController>,
    pub budget_service: Arc<BudgetController>,
    pub analytics_service: Arc<AnalyticsController>,
    pub category_service: Arc<CategoryController>,
    pub jwt: JwtState,
}

impl FromRef<AppState> for JwtState {
    fn from_ref(input: &AppState) -> Self {
        input.jwt.clone()
    }
}

impl AppState {
    pub fn setup<R: RepositoryFactory>(repo_factory: R, jwt_secret: Option<&str>) -> Self {
        let (user, account, category, transaction, budget) = repo_factory.all_dyn();

        Self {
            auth_service: Arc::new(AuthController::new(Arc::clone(&user))),
            account_service: Arc::new(AccountController::new(Arc::clone(&account))),
            transaction_service: Arc::new(TransactionController::new(
                Arc::clone(&transaction),
                Arc::clone(&account),
                Arc::clone(&budget),
                Arc::clone(&category),
            )),
            budget_service: Arc::new(BudgetController::new(
                Arc::clone(&budget),
                Arc::clone(&category),
                Arc::clone(&transaction),
            )),
            analytics_service: Arc::new(AnalyticsController::new(
                transaction,
                Arc::clone(&category),
            )),
            category_service: Arc::new(CategoryController::new(budget, category)),
            jwt: jwt_secret
                .map(|s| JwtState::from_secret(s.as_bytes()))
                .unwrap_or_else(|| JwtState::from_env()),
        }
    }
}
