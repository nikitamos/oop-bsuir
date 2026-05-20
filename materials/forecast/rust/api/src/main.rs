use std::{error::Error, path::PathBuf};

use api::{WeatherHandler, weather};
use shared::utils::getenv;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(info(title = "Whether Example API", version = "1.0"))]
struct ApiDoc;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve())
}

async fn serve() -> anyhow::Result<()> {
    let (router, doc) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // .nest("/api/v1", weather::routers())
        .nest("/api/v2", api::routes_v2(WeatherHandler::new()))
        .split_for_parts();
    let router = router
        .merge(SwaggerUi::new("/swagger/index.html").url("/api-docs/openapi.json", doc.clone()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    let docs_dir: PathBuf = getenv("OPENAPI_DOCS_DIR", "./docs/".to_string()).into();
    tokio::fs::create_dir_all(&docs_dir).await?;
    tokio::try_join!(
        tokio::fs::write(docs_dir.join("swagger.json"), doc.to_pretty_json()?),
        tokio::fs::write(docs_dir.join("swagger.yaml"), doc.to_yaml()?),
    )?;
    axum::serve(listener, router).await?;
    Ok(())
}
