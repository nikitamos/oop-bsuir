use chrono::NaiveDate;

use rust_decimal::Decimal;

pub struct Budget {
    pub id: i64,
    pub user_id: i64,
    pub category_id: i64,
    pub amount_limit: Decimal,
    pub month: NaiveDate,
    pub current_spent: Decimal,
}
