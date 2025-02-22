use actix_jwt_auth_middleware::TokenSigner;
use actix_web::{post, get, web, Responder};
use actix_web::middleware::from_fn;
use jwt_compact::alg::Hs256;
use sea_orm::DatabaseConnection;
use crate::dtos::user_dto::UserLogin;
use crate::entities::users;
use crate::services::{hash_service, user_service};
use crate::services::auth_service::{is_registered, Role, UserClaims};

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(login)
            .service(register_user)
            .service(
                web::scope("")
                    .wrap(from_fn(is_registered))
                    .service(get_current_user)
            )
    );
}

#[post("/register")]
pub async fn register_user(
    db: web::Data<DatabaseConnection>, 
    new_user: web::Json<users::Model>
) -> impl Responder {
    let password = new_user.password.clone();
    let hashed_password =  match hash_service::hash_password(&password.unwrap_or_default()).await {
        Ok(hashed_password) => hashed_password,
        Err(error) => return error
    };

    let mut user = new_user.into_inner();
    user.password = Some(hashed_password);

    user_service::create_user(db, web::Json(user)).await
}

#[post("/login")]
pub async fn login(
    db: web::Data<DatabaseConnection>,
    user_login: web::Json<UserLogin>,
    token_signer: web::Data<TokenSigner<UserClaims, Hs256>>,
) -> impl Responder {
    user_service::login(db, user_login, token_signer, Role::RegisteredUser).await
}

#[get("/current")]
pub async fn get_current_user(
    db: web::Data<DatabaseConnection>,
    user_claims: UserClaims
) -> impl Responder {
    user_service::get_user(db, user_claims.id).await
}