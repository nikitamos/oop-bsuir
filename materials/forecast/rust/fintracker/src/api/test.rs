use axum_test::TestServer;
use rstest::{fixture, rstest};
use sea_orm::{Database, DatabaseConnection};

use crate::{api::state::AppState, clients::sea_orm::{SeaOrmRepositoryFactory, setup_db}, repositories::RepositoryFactory};

const TEST_DB_CONNECTION_STRING: &str = "sqlite::memory:";

#[fixture]
async fn db() -> DatabaseConnection {
    let db = Database::connect(TEST_DB_CONNECTION_STRING).await.unwrap();
    setup_db(&db).await.unwrap();
    db
}

#[fixture]
async fn repo_factory(#[future] db: DatabaseConnection) -> impl RepositoryFactory {
    SeaOrmRepositoryFactory::new(db.await)
}

#[fixture]
async fn app<RF: RepositoryFactory>(#[future] repo_factory: RF) -> TestServer {
    let state = AppState::setup(repo_factory.await);
    TestServer::new(crate::api::routes(state))
}

#[rstest]
#[tokio::test]
async fn test_some_login(#[future] app: TestServer) {
    let app = app.await;
}
