use actix_web::{get, post, web, Responder};
use actix_web::middleware::from_fn;
use sea_orm::DatabaseConnection;
use crate::dtos::group_dto::CreateGroupForm;
use crate::services::auth_service::is_registered;
use crate::services::group_service;
use crate::services::hash_service::hash_password;

pub fn group_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/groups")
            .service(list_groups)
            .service(create_group)
            .service(
                web::scope("")
                    .wrap(from_fn(is_registered))
                    .service(list_group_videos)
            )
    );
}

#[get("")]
pub async fn list_groups(
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    group_service::get_groups(db.clone()).await
}

#[post("")]
pub async fn create_group(
    db: web::Data<DatabaseConnection>,
    form: web::Json<CreateGroupForm>,
) -> impl Responder {
    let form = form.into_inner();

    let hashed_password = match hash_password(&form.password.unwrap_or_default()).await {
        Ok(hashed_password) => hashed_password,
        Err(error) => return error
    };

    let hashed_form = CreateGroupForm {
        name: form.name.clone(),
        password: Some(hashed_password),
    };

    group_service::create_group(db, hashed_form).await
}

#[get("/{group_id}/videos")]
pub async fn list_group_videos(
    db: web::Data<DatabaseConnection>,
    group_id: web::Path<i64>,
) -> impl Responder {
    group_service::get_group_videos(db, group_id).await
}