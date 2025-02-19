mod db;
mod entities;
mod services;
mod endpoints;
mod dtos;

use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_jwt_auth_middleware::use_jwt::UseJWTOnApp;
use actix_web::{web, App, HttpServer};
use jwt_compact::alg::{Hs256, Hs256Key};
use crate::endpoints::user_endpoints::user_routes;
use crate::services::auth_service::UserClaims;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    services::hash_service::init().await;
    let db = db::establish_connection().await
        .expect("Failed to establish database connection");
    
    HttpServer::new(move || {
        let public_key = Hs256Key::new(std::env::var("JWT_PUBLIC_KEY").unwrap().to_string().into_bytes());
        let private_key = Hs256Key::new(std::env::var("JWT_PRIVATE_KEY").unwrap().to_string().into_bytes());
        
        let authority = Authority::<UserClaims, Hs256, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(private_key)
                    .algorithm(Hs256)
                    .build()
                    .expect("Failed to build Authority!"),
            ))
            .verifying_key(public_key)
            .build()
            .expect("Failed to build Authority!");
        
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(user_routes)
            .use_jwt(authority.clone(), web::scope(""))
    })
    .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
