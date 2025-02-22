use actix_jwt_auth_middleware::TokenSigner;
use actix_web::{delete, get, post, put, web, Responder};
use actix_web::middleware::from_fn;
use jwt_compact::alg::Hs256;
use sea_orm::DatabaseConnection;
use crate::dtos::user_dto::UserLogin;
use crate::services::auth_service::{is_admin, Role, UserClaims};
use crate::services::user_service;
use crate::services::user_service::{UserOperation};

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .service(admin_login)
            .service(
                web::scope("")
                    .wrap(from_fn(is_admin))
                    .service(get_all_users)
                    .service(delete_user)
                    .service(restore_user)
            )
    );
}

#[post("/login")]
pub async fn admin_login(
    db: web::Data<DatabaseConnection>,
    user_login: web::Json<UserLogin>,
    token_signer: web::Data<TokenSigner<UserClaims, Hs256>>,
) -> impl Responder {
    user_service::login(db, user_login, token_signer, Role::Admin).await
}

#[get("/users")]
pub async fn get_all_users(db: web::Data<DatabaseConnection>) -> impl Responder {
    user_service::get_users(db).await
}

#[delete("/user/{id}")]
pub async fn delete_user(db: web::Data<DatabaseConnection>, id: web::Path<i64>) -> impl Responder {
    user_service::modify_user_state(db, id, UserOperation::Delete).await
}

#[put("/user/{id}")]
pub async fn restore_user(db: web::Data<DatabaseConnection>, id: web::Path<i64>) -> impl Responder {
    user_service::modify_user_state(db, id, UserOperation::Restore).await
}