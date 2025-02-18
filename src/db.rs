use sea_orm::{ Database, DatabaseConnection, DbErr };
use tokio_retry::Retry;
use tokio_retry::strategy::{ ExponentialBackoff };

pub async fn establish_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let strategy = ExponentialBackoff::from_millis(10).take(10);

    Retry::spawn(strategy, || async { Database::connect(&database_url).await }).await
}
