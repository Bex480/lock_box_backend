use actix_jwt_auth_middleware::TokenSigner;
use actix_web::{post, get, web, Responder};
use actix_web::middleware::from_fn;
use jwt_compact::alg::Hs256;
use sea_orm::DatabaseConnection;
use crate::dtos::group_dto::JoinGroup;
use crate::dtos::user_dto::{UserLogin, UserRegister};
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
                    .service(join_group)
            )
    );
}

#[post("/register")]
pub async fn register_user(
    db: web::Data<DatabaseConnection>, 
    new_user: web::Json<UserRegister>
) -> impl Responder {
    let password = new_user.password.clone();
    let hashed_password =  match hash_service::hash_password(&password).await {
        Ok(hashed_password) => hashed_password,
        Err(error) => return error
    };

    let mut user = new_user.into_inner();
    user.password = hashed_password;

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

#[post("/join/group/{group_id}")]
pub async fn join_group(
    db: web::Data<DatabaseConnection>,
    group_id: web::Path<i64>,
    join_group: web::Json<JoinGroup>,
    user_claims: UserClaims
) -> impl Responder {
    user_service::join_group(db, group_id, join_group, user_claims).await
}