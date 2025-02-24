mod db;
mod entities;
mod services;
mod endpoints;
mod dtos;

use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_jwt_auth_middleware::use_jwt::UseJWTOnApp;
use actix_web::{ web, App, HttpServer};
use jwt_compact::alg::{Hs256, Hs256Key};
use core::time::Duration;
use actix_multipart::form::MultipartFormConfig;
use actix_web::middleware::{Logger};
use env_logger::Env;
use crate::endpoints::admin_endpoints::{admin_routes};
use crate::endpoints::storage_endpoints::storage_routes;
use crate::endpoints::user_endpoints::{user_routes};
use crate::services::auth_service::{UserClaims};
use crate::services::storage_service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    services::hash_service::init().await;
    let db = db::establish_connection().await
        .expect("Failed to establish database connection");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let s3_client = storage_service::create_client().await;
    
    HttpServer::new(move || {
        let public_key = Hs256Key::new(std::env::var("JWT_PUBLIC_KEY").unwrap_or_default().to_string().into_bytes());
        let private_key = Hs256Key::new(std::env::var("JWT_PRIVATE_KEY").unwrap_or_default().to_string().into_bytes());
        
        let authority = Authority::<UserClaims, Hs256, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(private_key.clone())
                    .algorithm(Hs256)
                    .access_token_lifetime(Duration::new(60, 0))
                    .refresh_token_lifetime(Duration::new(3600, 0))
                    .build()
                    .expect("Failed to build Authority!"),
            ))
            .enable_cookie_tokens(true)
            .renew_access_token_automatically(true)
            .verifying_key(public_key)
            .build()
            .expect("Failed to build Authority!");
        
        App::new()
            .wrap(Logger::default())
            .app_data(
                MultipartFormConfig::
                total_limit(Default::default(), 1024*1024*100).memory_limit(1024*1024*5)
            )
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(s3_client.clone()))
            .configure(user_routes)
            .configure(admin_routes)
            .configure(storage_routes)
            .use_jwt(authority.clone(), web::scope(""))

    })
    .bind(("127.0.0.1", 8080))?
        .run()
        .await
}