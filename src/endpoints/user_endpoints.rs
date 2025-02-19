use actix_jwt_auth_middleware::TokenSigner;
use actix_web::{post, get, web, Responder, delete, put};
use jwt_compact::alg::Hs256;
use sea_orm::DatabaseConnection;
use crate::dtos::user_dto::UserLogin;
use crate::entities::users;
use crate::services::{hash_service, user_service};
use crate::services::auth_service::UserClaims;
use crate::services::user_service::{UserOperation};

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register_user)
        .service(get_all_users)
        .service(get_user_by_id)
        .service(delete_user)
        .service(restore_user)
        .service(login);
}

#[post("/users")]
pub async fn register_user(
    db: web::Data<DatabaseConnection>, 
    new_user: web::Json<users::Model>
) -> impl Responder {
    let password = new_user.password.clone();
    let hashed_password =  match hash_service::hash_password(&password.unwrap()).await {
        Ok(hashed_password) => hashed_password,
        Err(error) => return error
    };

    let mut user = new_user.into_inner();
    user.password = Some(hashed_password);

    user_service::create_user(db, web::Json(user)).await
}

#[get("/users")]
pub async fn get_all_users(db: web::Data<DatabaseConnection>) -> impl Responder {
    user_service::get_users(db).await
}

#[get("/users/{id}")]
pub async fn get_user_by_id(
    db: web::Data<DatabaseConnection>, 
    id: web::Path<i64>
) -> impl Responder {
    user_service::get_user(db, id).await
}

#[post("/users/login")]
pub async fn login(
    db: web::Data<DatabaseConnection>, 
    user_login: web::Json<UserLogin>,
    token_signer: web::Data<TokenSigner<UserClaims, Hs256>>,
) -> impl Responder {
    user_service::login(db, user_login, token_signer).await
}

#[delete("/users/{id}")]
pub async fn delete_user(db: web::Data<DatabaseConnection>, id: web::Path<i64>) -> impl Responder {
    user_service::modify_user_state(db, id, UserOperation::Delete).await
}

#[put("/users/{id}")]
pub async fn restore_user(db: web::Data<DatabaseConnection>, id: web::Path<i64>) -> impl Responder {
    user_service::modify_user_state(db, id, UserOperation::Restore).await
}