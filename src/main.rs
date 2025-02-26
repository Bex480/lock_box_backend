mod db;
mod entities;
mod services;
mod endpoints;
mod dtos;

use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_jwt_auth_middleware::use_jwt::{UseJWTOnApp, UseJWTOnScope};
use actix_web::{web, HttpResponse, Responder};
use jwt_compact::alg::{Hs256, Hs256Key};
use core::time::Duration;
use actix_multipart::form::MultipartFormConfig;
use actix_web::web::ServiceConfig;
use shuttle_actix_web::ShuttleActixWeb;
use crate::endpoints::admin_endpoints::{admin_routes};
use crate::endpoints::group_endpoints::group_routes;
use crate::endpoints::storage_endpoints::storage_routes;
use crate::endpoints::user_endpoints::{user_routes};
use crate::services::auth_service::{UserClaims};
use crate::services::storage_service;
use shuttle_runtime::SecretStore;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    services::hash_service::init().await;
    let db = db::establish_connection(secrets.clone()).await
        .expect("Failed to establish database connection");

    let s3_client = storage_service::create_client(secrets.clone()).await;

    std::env::set_var("JWT_PRIVATE_KEY", secrets.get("JWT_PRIVATE_KEY").unwrap_or_default().to_string());
    std::env::set_var("VIDEO_STORAGE_BUCKET", secrets.get("VIDEO_STORAGE_BUCKET").unwrap_or_default().to_string());

    let public_key = Hs256Key::new(secrets.get("JWT_PUBLIC_KEY").unwrap_or_default().into_bytes());
    let private_key = Hs256Key::new(secrets.get("JWT_PRIVATE_KEY").unwrap_or_default().into_bytes());

    let authority = Authority::<UserClaims, Hs256, _, _>::new()
        .refresh_authorizer(|| async move { Ok(()) })
        .token_signer(Some(
            TokenSigner::new()
                .signing_key(private_key.clone())
                .algorithm(Hs256)
                .access_token_lifetime(Duration::new(3600, 0))
                .refresh_token_lifetime(Duration::new(3600*24, 0))
                .build()
                .expect("Failed to build Authority!"),
        ))
        .enable_cookie_tokens(true)
        .renew_access_token_automatically(true)
        .verifying_key(public_key)
        .build()
        .expect("Failed to build Authority!");

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(
                MultipartFormConfig::
                total_limit(Default::default(), 1024 * 1024 * 512).memory_limit(1024 * 1024 * 5)
            )
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(s3_client.clone()))
            .service(
                web::scope("")
                    .configure(user_routes)
                    .configure(admin_routes)
                    .configure(storage_routes)
                    .configure(group_routes)
                    .use_jwt(authority.clone(), web::scope(""))
            );
    };

    Ok(config.into())
}