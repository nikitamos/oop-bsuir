use chrono::NaiveDate;

use rust_decimal::Decimal;

pub struct Budget {
    pub id: i64,
    pub user_id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub amount_limit: Decimal,
    pub month: NaiveDate,
    pub current_spent: Decimal,
}

impl Budget {
    pub fn usage_percentage(&self) -> u32 {
        todo!()
    }
    pub fn is_limit_exceeded(&self) -> bool {
        self.current_spent > self.amount_limit
    }
}
