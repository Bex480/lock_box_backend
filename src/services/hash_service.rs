use actix_web::HttpResponse;
use argon2_async::{hash, set_config, verify, Config};

pub async fn init() {
    set_config(Config::default()).await;
}

pub async fn hash_password(password: &str) -> Result<String, HttpResponse> {
   hash(password).await.map_err(|_| HttpResponse::InternalServerError().finish())
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, HttpResponse> {
    verify(password.to_string(), hash.to_string()).await.map_err(|_| HttpResponse::InternalServerError().finish())
}