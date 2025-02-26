use sea_orm::{ Database, DatabaseConnection, DbErr };
use shuttle_runtime::SecretStore;
use tokio_retry::Retry;
use tokio_retry::strategy::{ ExponentialBackoff };

pub async fn establish_connection(secrets: SecretStore) -> Result<DatabaseConnection, DbErr> {

    let database_url = secrets.get("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let strategy = ExponentialBackoff::from_millis(10).take(10);

    Retry::spawn(strategy, || async { Database::connect(&database_url).await }).await
}
