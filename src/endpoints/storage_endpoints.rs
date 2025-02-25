use actix_multipart::form::MultipartForm;
use actix_web::{get, post, web, HttpResponse};
use aws_sdk_s3 as s3;
use sea_orm::DatabaseConnection;
use crate::services::storage_service;
use crate::services::storage_service::UploadForm;

pub fn storage_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/storage")
            .service(upload_file)
            .service(playback)
    );
}

#[post("/upload/video/{group_id}")]
pub async fn upload_file(
    client: web::Data<s3::Client>,
    MultipartForm(form): MultipartForm<UploadForm>,
    db: web::Data<DatabaseConnection>,
    group_id: web::Path<i64>,
) -> HttpResponse {
    storage_service::upload_video(client, MultipartForm(form), db, group_id).await
        .unwrap_or(HttpResponse::InternalServerError().finish())
}

#[get("/playback/{key}")]
pub async fn playback(client: web::Data<s3::Client>, key: web::Path<String>) -> HttpResponse {
    storage_service::serve_video(client, key).await
        .unwrap_or(HttpResponse::InternalServerError().body("OUTER"))
}