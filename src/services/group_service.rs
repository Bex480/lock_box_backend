use actix_web::{error, web, HttpResponse};
use aws_sdk_s3::types::Type::Group;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use crate::dtos::group_dto::CreateGroupForm;
use crate::entities;
use crate::entities::{group_video, groups, videos};
use crate::entities::prelude::{Groups, GroupVideo, Videos};

pub async fn create_group(
    db: web::Data<DatabaseConnection>,
    form: CreateGroupForm,
) -> HttpResponse {
    let db = db.as_ref();

    let group = groups::ActiveModel {
        name: Set(form.name.clone()),
        password: Set(form.password.clone()),
        ..Default::default()
    };

    match group.insert(db).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().body("Error inserting group"),
    }
}

pub async fn get_groups(db: web::Data<DatabaseConnection>, ) -> HttpResponse {
    let groups = groups::Entity::find().all(db.as_ref()).await.unwrap_or_default();
    HttpResponse::Ok().json(groups)
}

pub async fn get_group_videos(
    db: web::Data<DatabaseConnection>,
    group_id: web::Path<i64>,
) -> HttpResponse {
    let db = db.as_ref();
    let group_id = group_id.into_inner();

    let entries = GroupVideo::find()
        .filter(group_video::Column::GroupId.eq(group_id))
        .all(db)
        .await
        .unwrap_or_default();

    let videos = entries.load_one(Videos, db).await.expect("Error loading videos");

    HttpResponse::Ok().json(videos)
}

pub async fn add_video_to_group(
    group_id: i64,
    video_id: i64,
    db: web::Data<DatabaseConnection>
) -> Result<(), actix_web::Error> {
    let db = db.as_ref();

    let entity = group_video::ActiveModel {
        group_id: Set(group_id),
        video_id: Set(video_id),
        ..Default::default()
    };

    match entity.insert(db).await {
        Ok(_) => Ok(()),
        Err(_) => Err(error::ErrorInternalServerError("Failed to add video to group!")),
    }
}

pub async fn get_group_users() {
    todo!()
}