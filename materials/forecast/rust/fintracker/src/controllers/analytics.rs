use chrono::NaiveDateTime;

use crate::{
    models::{
        domain::budget::Budget,
        dtos::{ExpensesSummary, MonthlySummary},
    },
    repositories::*,
    shared::error::Result,
};
use std::sync::Arc;

pub struct AnalyticsController {
    transaction_repo: Arc<dyn TransactionRepository>,
    category_repo: Arc<dyn CategoryRepository>,
}

impl AnalyticsController {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        category_repo: Arc<dyn CategoryRepository>,
    ) -> Self {
        todo!()
    }

    /// Сводка расходов за период в разрезе категорий
    pub async fn get_expenses_summary(
        &self,
        user_id: i64, // хотя репозиторий может требовать список счетов
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<ExpensesSummary> {
        todo!()
    }

    /// Доходы vs Расходы по месяцам
    pub async fn get_income_vs_expenses(
        &self,
        user_id: i64,
        months: u32,
    ) -> Result<Vec<MonthlySummary>> {
        todo!()
    }
}
