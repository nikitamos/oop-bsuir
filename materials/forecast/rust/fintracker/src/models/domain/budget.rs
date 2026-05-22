use chrono::NaiveDate;

use rust_decimal::{Decimal, dec, prelude::ToPrimitive};

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
    pub fn is_limit_exceeded(&self) -> bool {
        self.current_spent > self.amount_limit
    }
    pub fn record_expense(&mut self, amount: Decimal) {
        self.current_spent += amount;
    }

    pub fn is_over_limit(&self) -> bool {
        self.current_spent > self.amount_limit
    }

    pub fn usage_percentage(&self) -> u32 {
        if self.amount_limit == dec!(0) {
            return 0;
        }
        ((self.current_spent / self.amount_limit) * dec!(100))
            .to_u32()
            .unwrap()
    }

    pub fn remaining(&self) -> Decimal {
        (self.amount_limit - self.current_spent).max(dec!(0))
    }
    pub fn get_warning(&self, cat_name: Option<&str>) -> Option<String> {
        if self.is_over_limit() {
            Some(format!(
                "Budget exceeded for category '{}' (id={}): spent {:.2} of {:.2} ({}% used)",
                cat_name.unwrap_or("<name unavailable>"),
                self.category_id,
                self.current_spent,
                self.amount_limit,
                self.usage_percentage()
            ))
        } else {
            None
        }
    }
}
