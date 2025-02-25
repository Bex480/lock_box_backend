use std::ops::Deref;
use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_web::{error, web, Error, HttpMessage, HttpRequest, HttpResponse};

use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3 as s3;
use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
use aws_sdk_s3::types::{ChecksumMode, CompletedMultipartUpload, CompletedPart};
use futures_util::stream::StreamExt;
use aws_smithy_types::byte_stream::{ByteStream, Length};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use crate::entities::videos;
use crate::services;
use crate::services::group_service;

pub async fn create_client() -> s3::Client {
    dotenv::dotenv().ok();
    let access_token_id = std::env::var("AWS_ACCESS_KEY_ID").expect("ACCESS_TOKEN_ID");
    let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY").expect("SECRET_ACCESS_KEY");
    let endpoint_url = std::env::var("AWS_ENDPOINT_URL").expect("ENDPOINT_URL");

    let credentials = Credentials::new(
        access_token_id,
        secret_access_key,
        None,
        None,
        "custom"
    );

    let base_config = aws_config::from_env()
        .region(Region::new("auto"))
        .load()
        .await;

    let s3_config = s3::config::Builder::from(&base_config)
        .credentials_provider(credentials)
        .endpoint_url(endpoint_url)
        .region(Region::new("auto"))
        .build();

    s3::Client::from_conf(s3_config)
}

pub fn generate_random_key(file_extension: &str) -> String {
    let random_str = nanoid::nanoid!(10);
    format!("{}.{}", random_str, file_extension)
}

pub fn extract_file_extension(req_data: &HttpRequest) -> Option<String> {
    todo!()
}

pub async fn serve_video(
    client: web::Data<s3::Client>,
    key: web::Path<String>,
) -> Result<HttpResponse, Error> {
    dotenv::dotenv().ok();

    let bucket_name = std::env::var("VIDEO_STORAGE_BUCKET").expect("BUCKET_NAME");

    let object = client.get_object()
        .checksum_mode(ChecksumMode::Enabled)
        .bucket(&bucket_name)
        .key(&key.into_inner())
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch video: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch video")
        })?;

    let body = match object.body.collect().await {
        Ok(body) => body,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("INNER ERROR"))
    };

    Ok(HttpResponse::Ok().content_type("video/mp4").body(body.into_bytes()))
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "512 MiB")]
    file: TempFile,
}

const CHUNK_SIZE: u64 = 1024 * 1024 * 5;
const MAX_CHUNKS: u64 = 10000;

pub async fn upload_video(
    client: web::Data<s3::Client>,
    MultipartForm(form): MultipartForm<UploadForm>,
    db: web::Data<DatabaseConnection>,
    group_id: web::Path<i64>,
) -> Result<HttpResponse, Error> {

    let bucket_name = std::env::var("VIDEO_STORAGE_BUCKET").expect("BUCKET_NAME");
    let key = generate_random_key("mp4");

    let video = videos::ActiveModel {
        name: Set(form.file.file_name.unwrap_or_default().clone()),
        key: Set(key.clone()),
        ..Default::default()
    };

    let inserted_video = match video.insert(db.as_ref()).await {
        Ok(video) => video,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to insert video!")),
    };

    group_service::add_video_to_group(group_id.into_inner(), inserted_video.id, db.clone()).await?;

    let multipart_upload_res: CreateMultipartUploadOutput = client
        .create_multipart_upload()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| {
            eprintln!("{:?}", e);
            error::ErrorInternalServerError("Failed to create multipart upload")
        })?;

    let upload_id = multipart_upload_res.upload_id()
        .ok_or(error::ErrorInternalServerError("Missing upload_id after CreateMultipartUpload"))?;

    let form_file_size = &form.file.size;
    let file_size = form_file_size.to_owned() as u64;

    let mut chunk_count = (file_size / CHUNK_SIZE) + 1;
    let mut size_of_last_chunk = file_size % CHUNK_SIZE;
    if size_of_last_chunk == 0 {
        size_of_last_chunk = CHUNK_SIZE;
        chunk_count -= 1;
    }

    if file_size == 0 {
        return Err(error::ErrorInternalServerError("Bad file size."));
    }
    if chunk_count > MAX_CHUNKS {
        return Err(error::ErrorBadRequest("Too many chunks!" ));
    }

    let mut upload_parts: Vec<aws_sdk_s3::types::CompletedPart> = Vec::new();

    for chunk_index in 0..chunk_count {
        let this_chunk = if chunk_count - 1 == chunk_index {
            size_of_last_chunk
        } else {
            CHUNK_SIZE
        };
        let stream = ByteStream::read_from()
            .path(form.file.file.path())
            .offset(chunk_index * CHUNK_SIZE)
            .length(Length::Exact(this_chunk))
            .build()
            .await
            .unwrap();

        let part_number = (chunk_index as i32) + 1;
        let upload_part_res = client
            .upload_part()
            .key(&key)
            .bucket(&bucket_name)
            .upload_id(upload_id)
            .body(stream)
            .part_number(part_number)
            .send()
            .await
            .map_err(|e| {
                eprintln!("{:?}", e);
                error::ErrorInternalServerError("Failed to upload part")
            })?;

        upload_parts.push(
            CompletedPart::builder()
                .e_tag(upload_part_res.e_tag.unwrap_or_default())
                .part_number(part_number)
                .build(),
        );
    }

    let completed_multipart_upload: CompletedMultipartUpload = CompletedMultipartUpload::builder()
        .set_parts(Some(upload_parts))
        .build();

    let _complete_multipart_upload_res = client
        .complete_multipart_upload()
        .bucket(&bucket_name)
        .key(&key)
        .multipart_upload(completed_multipart_upload)
        .upload_id(upload_id)
        .send()
        .await
        .map_err(|e| {
            eprintln!("{:?}", e);
            error::ErrorInternalServerError("Failed to complete multipart upload")
        })?;

    Ok(HttpResponse::Ok().body("Upload completed successfully!"))
}