mod db;
mod entities;
mod services;
mod endpoints;
mod dtos;

use actix_jwt_auth_middleware::{AuthenticationService, Authority, TokenSigner};
use actix_jwt_auth_middleware::use_jwt::UseJWTOnApp;
use actix_web::{web, App, HttpServer};
use jwt_compact::alg::{Hs256, Hs256Key};
use core::time::Duration;
use crate::endpoints::user_endpoints::{user_routes};
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
            .app_data(web::Data::new(db.clone()))
            .configure(user_routes)
            .use_jwt(authority.clone(), web::scope(""))
            .service(
                web::scope("")
                    .wrap(
                        AuthenticationService::new(authority.clone())
                    )
            )
    })
    .bind(("127.0.0.1", 8080))?
        .run()
        .await
}