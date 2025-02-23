use actix_web::{get, post, web, HttpRequest, HttpResponse};
use aws_sdk_s3 as s3;
use crate::services::storage_service;

pub fn storage_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/storage")
            .service(upload_file)
            .service(playback)
    );
}

#[post("/upload/video")]
pub async fn upload_file(
    client: web::Data<s3::Client>,
    payload: web::Payload,
    request: HttpRequest
) -> HttpResponse {
    storage_service::upload_video(client, request, payload).await
        .unwrap_or(HttpResponse::InternalServerError().finish())
}

#[get("/playback/{key}")]
pub async fn playback(client: web::Data<s3::Client>, key: web::Path<String>) -> HttpResponse {
    storage_service::serve_video(client, key).await
        .unwrap_or(HttpResponse::InternalServerError().body("OUTER"))
}