use fintracker::clients::sea_orm::{SeaOrmRepositoryFactory, setup_db};
use sea_orm::Database;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::filter::FilterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, filter};
use utoipa::OpenApi;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

use fintracker::{api, clients};

#[derive(OpenApi)]
#[openapi(
    paths(
        api::register,
        api::login,
        api::create_account,
        api::get_accounts,
        api::create_transaction,
        api::get_transactions,
        api::set_budget,
        api::get_budgets,
        api::get_budget_status,
        api::get_expenses_summary,
        api::get_monthly_summary,
        api::get_categories,
        api::create_category
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "accounts", description = "Account management"),
        (name = "transactions", description = "Income and expense transactions"),
        (name = "budgets", description = "Monthly budget limits"),
        (name = "analytics", description = "Spending analytics and reports"),
        (name = "categories", description = "Transaction categories reference"),
    ),
    info(
        title = "Personal Finance Tracker API",
        version = "1.0.0",
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            let mut http_auth = Http::new(HttpAuthScheme::Bearer);
            http_auth.bearer_format = Some("JWT token issued by `auth` APIs".to_string());
            let scheme = SecurityScheme::Http(http_auth);
            components.add_security_scheme("jwt", scheme)
        }
    }
}

fn setup_tracing() {
    use tracing_subscriber::fmt;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "sqlx=warn,sea_orm=warn,tower_http=debug,{}=debug",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(fmt::layer().event_format(fmt::format().without_time()))
        .init();
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    setup_tracing();

    dotenvy::dotenv()?;
    let db = Database::connect(
        std::env::var("DB_CONNECTION").expect("DB_CONNECTION environment variable not set"),
    )
    .await?;
    setup_db(&db).await?;
    let repo_factory = SeaOrmRepositoryFactory::new(db);

    let state = api::state::AppState::setup(repo_factory, None);

    let doc = <ApiDoc as OpenApi>::openapi();
    let app = api::routes(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", doc))
        .with_state(())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server running on http://localhost:3000");
    tracing::info!("Swagger UI: http://localhost:3000/swagger-ui");
    axum::serve(listener, app).await?;
    Ok(())
}
