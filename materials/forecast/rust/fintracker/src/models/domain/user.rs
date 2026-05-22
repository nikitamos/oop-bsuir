use chrono::NaiveDateTime;

pub struct User {
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}
