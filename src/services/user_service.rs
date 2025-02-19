use actix_jwt_auth_middleware::{AuthError, AuthResult, TokenSigner};
use actix_web::{web, HttpResponse};
use jwt_compact::alg::Hs256;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, IntoActiveModel};
use sea_orm::ActiveValue::Set;
use crate::dtos::user_dto::{UserLogin, UserResponse};
use crate::entities::users;
use crate::services::auth_service::{Role, UserClaims};
use crate::services::hash_service;

pub async fn create_user(
    db: web::Data<DatabaseConnection>,
    new_user: web::Json<users::Model>
) -> HttpResponse {
    let db = db.get_ref();
    let user = users::ActiveModel {
        username: Set(new_user.username.clone()),
        email: Set(new_user.email.clone()),
        password: Set(new_user.password.clone()),
        ..Default::default()
    };
    
    match user.insert(db).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_users(db: web::Data<DatabaseConnection>) -> HttpResponse {
    let db = db.get_ref();
    match users::Entity::find().all(db).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_user(db: web::Data<DatabaseConnection>, user_id: web::Path<i64>) -> HttpResponse {
    let db = db.get_ref();
    let user_id = user_id.into_inner();

    match users::Entity::find().filter(users::Column::Id.eq(user_id)).one(db).await {
        Ok(Some(user)) => {
            let response = UserResponse {
                username: user.username.unwrap(),
                email: user.email,
            };
            HttpResponse::Ok().json(response)
        },
        Ok(None) => HttpResponse::Ok().body(user_id.to_string()),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn login(
    db: web::Data<DatabaseConnection>,
    user_login: web::Json<UserLogin>,
    token_signer: web::Data<TokenSigner<UserClaims, Hs256>>,
    user_role: Role
) -> AuthResult<HttpResponse> {
    let db = db.get_ref();

    let user_result = users::Entity::find()
        .filter(users::Column::Email.eq(user_login.email.clone()))
        .one(db)
        .await;

    match user_result {
        Ok(Some(user)) => {
            if user.is_deleted { return Ok(HttpResponse::Unauthorized().finish()); }
            
            let user_claim: UserClaims = UserClaims {
                id: user.id,
                role: user_role,
            };
            
            match hash_service::verify_password(&user_login.password, &user.password.unwrap()).await {
                Ok(true) => {
                    Ok(HttpResponse::Ok()
                        .cookie(token_signer.create_access_cookie(&user_claim)?)
                        .cookie(token_signer.create_refresh_cookie(&user_claim)?)
                        .finish()
                    )
                },
                Err(_) => Ok(HttpResponse::Unauthorized().finish()),
                _ => Ok(HttpResponse::Unauthorized().finish()),
            }
        },
        Ok(None) => Ok(HttpResponse::Unauthorized().finish()),
        Err(_) => Ok(HttpResponse::Unauthorized().finish()),
    }
}

pub enum UserOperation {
    Delete,
    Restore,
}

pub async fn modify_user_state(
    db: web::Data<DatabaseConnection>,
    user_id: web::Path<i64>,
    operation: UserOperation
) -> HttpResponse {
    let db = db.get_ref();

    let result = users::Entity::find()
        .filter(users::Column::Id.eq(user_id.into_inner()))
        .one(db)
        .await;

    match result {
        Ok(Some(user)) => {
            let mut user = user.into_active_model();
            match operation {
                UserOperation::Delete => { user.is_deleted = Set(true); }
                UserOperation::Restore => { user.is_deleted = Set(false); }
            }
            match user.update(db).await {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}