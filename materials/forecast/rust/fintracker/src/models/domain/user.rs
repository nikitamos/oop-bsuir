use bcrypt::{BcryptError, DEFAULT_COST, hash};

#[derive(Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
}

impl User {
    pub fn new_with_raw_password(
        email: impl Into<String>,
        password: impl AsRef<str>,
    ) -> Result<Self, BcryptError> {
        let password_hash = hash(password.as_ref(), DEFAULT_COST)?;

        Ok(Self {
            id: 0,
            email: email.into(),
            password_hash,
        })
    }
}
