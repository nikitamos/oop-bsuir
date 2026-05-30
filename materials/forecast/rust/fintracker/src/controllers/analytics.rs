use chrono::{Datelike, NaiveDate};
use rust_decimal::{Decimal, dec, prelude::ToPrimitive};

use crate::{
    models::{
        domain::{
            budget::Budget,
            transaction::{Transaction, TransactionType},
        },
        dtos::analytics::{CategorySummary, ExpensesSummary, MonthlySummary},
    },
    repositories::*,
    shared::{
        DateTimeProvider, DefaultDateTimeProvider,
        error::{ApplicationError, Result},
    },
};
use std::{collections::HashMap, sync::Arc};

pub struct AnalyticsController<DT: DateTimeProvider = DefaultDateTimeProvider> {
    transaction_repo: Arc<dyn TransactionRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    dt: DT,
}
impl AnalyticsController {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        category_repo: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            transaction_repo,
            category_repo,
            dt: Default::default(),
        }
    }
}

impl<DT: DateTimeProvider> AnalyticsController<DT> {
    pub async fn get_expenses_summary(
        &self,
        user_id: i64,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<ExpensesSummary> {
        if from >= to {
            return Err(ApplicationError::BadRequest(
                "'from' date must be earlier than 'to' date".to_string(),
            ));
        }

        let categories = self.category_repo.find_all().await?;
        let category_map: HashMap<i64, String> =
            categories.into_iter().map(|c| (c.id, c.name)).collect();

        let transactions = self
            .transaction_repo
            .find_by_user_and_period(user_id, from, to)
            .await?;

        let expenses: Vec<&Transaction> = transactions
            .iter()
            .filter(|t| matches!(t.transaction_type, TransactionType::Expense))
            .collect();

        let mut category_totals: HashMap<i64, Decimal> = HashMap::new();
        let mut total_expenses = Decimal::ZERO;

        for expense in &expenses {
            let entry = category_totals
                .entry(expense.category_id)
                .or_insert(Decimal::ZERO);
            *entry += expense.amount;
            total_expenses += expense.amount;
        }

        let mut categories_summary: Vec<CategorySummary> = category_totals
            .into_iter()
            .map(|(category_id, total_amount)| {
                let category_name = category_map
                    .get(&category_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Unknown category {}", category_id));

                let percentage = if total_expenses > Decimal::ZERO {
                    (total_amount / total_expenses) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };

                CategorySummary {
                    category_id,
                    category_name,
                    total_amount,
                    percentage: percentage.to_u32(),
                }
            })
            .collect();

        categories_summary.sort_by(|a, b| {
            b.total_amount
                .partial_cmp(&a.total_amount)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        total_expenses = (total_expenses * dec!(100)).round() / dec!(100);

        Ok(ExpensesSummary {
            total_expenses,
            categories: categories_summary,
        })
    }
    pub async fn get_monthly_summary(
        &self,
        user_id: i64,
        months: u32,
    ) -> Result<Vec<MonthlySummary>> {
        self.get_income_vs_expenses(user_id, months).await
    }

    pub async fn get_income_vs_expenses(
        &self,
        user_id: i64,
        months: u32,
    ) -> Result<Vec<MonthlySummary>> {
        let today = self.dt.today();
        let mut monthly_summaries = Vec::new();

        let mut current_month = today;
        for _ in 0..(months - 1) {
            current_month = current_month
                .with_day(1)
                .and_then(|d| d.pred_opt())
                .unwrap_or(current_month)
                .with_day(1)
                .unwrap_or(current_month);
        }
        current_month = current_month.with_day(1).unwrap_or(current_month);

        for _ in 0..months {
            let month_start = current_month;
            let month_end = if month_start.month() == 12 {
                NaiveDate::from_ymd_opt(month_start.year() + 1, 1, 1).and_then(|d| d.pred_opt())
            } else {
                NaiveDate::from_ymd_opt(month_start.year(), month_start.month() + 1, 1)
                    .and_then(|d| d.pred_opt())
            }
            .ok_or_else(|| {
                ApplicationError::Internal("Failed to calculate month end".to_string())
            })?;

            let transactions = self
                .transaction_repo
                .find_by_user_and_period(user_id, month_start, month_end)
                .await?;

            let mut total_income = Decimal::ZERO;
            let mut total_expenses = Decimal::ZERO;

            for transaction in &transactions {
                match transaction.transaction_type {
                    TransactionType::Income => total_income += transaction.amount,
                    TransactionType::Expense => total_expenses += transaction.amount,
                }
            }

            total_income = (total_income * dec!(100)).round() / dec!(100);
            total_expenses = (total_expenses * dec!(100)).round() / dec!(100);
            let net = (total_income - total_expenses * dec!(100)).round() / dec!(100);

            monthly_summaries.push(MonthlySummary {
                month: month_start,
                total_income,
                total_expenses,
                net,
            });

            current_month = if month_start.month() == 12 {
                NaiveDate::from_ymd_opt(month_start.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(month_start.year(), month_start.month() + 1, 1)
            }
            .unwrap_or(month_start);
        }

        Ok(monthly_summaries)
    }
}
