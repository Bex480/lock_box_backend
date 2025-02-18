mod db;
mod entities;
mod services;
mod endpoints;
mod dtos;

use actix_web::{web, App, HttpServer};
use crate::endpoints::user_endpoints::user_routes;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    services::hash_service::init().await;
    let db = db::establish_connection().await
        .expect("Failed to establish database connection");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(user_routes)
    })
    .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
