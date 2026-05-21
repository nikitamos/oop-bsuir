use utoipa::openapi::{RefOr, schema::*};

pub fn date_yyyy_mm_dd() -> impl Into<RefOr<Schema>> {
    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::String)
        .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
            "date".to_string(),
        )))
        .pattern(Some(r"^\d{4}-\d{2}-\d{2}$".to_string()))
        .examples(Some("2024-01-15".to_string()))
        .description(Some("Date in YYYY-MM-DD format"))
        .build()
}
