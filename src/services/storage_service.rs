use actix_web::{web, Error, HttpMessage, HttpRequest, HttpResponse};
use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3 as s3;
use aws_sdk_s3::primitives::{AggregatedBytes, ByteStream};
use futures_util::stream::StreamExt;

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

pub async fn upload_video(
    client: web::Data<s3::Client>,
    request: HttpRequest,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    dotenv::dotenv().ok();

    let mut bytes = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item?);
    }
    let body = ByteStream::from(bytes.freeze());

    let bucket_name = std::env::var("VIDEO_STORAGE_BUCKET").expect("BUCKET_NAME");
    let file_key = generate_random_key("mp4");

    client.put_object()
        .bucket(&bucket_name)
        .key(&file_key)
        .body(body)
        .content_type("video/mp4")
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to upload file: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to upload file")
        })?;

    Ok(HttpResponse::Ok().body(format!("File uploaded successfully! Key: {}", file_key)))
}

pub async fn serve_video(
    client: web::Data<s3::Client>,
    key: web::Path<String>,
) -> Result<HttpResponse, Error> {
    dotenv::dotenv().ok();

    let bucket_name = std::env::var("VIDEO_STORAGE_BUCKET").expect("BUCKET_NAME");

    let object = client.get_object()
        .bucket(&bucket_name)
        .key(&key.into_inner())
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch video: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch video")
        })?;

    println!("Object metadata: {:?}", object.metadata());
    println!("Content length: {:?}", object.content_length());

    let body = match object.body.collect().await {
        Ok(body) => body,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("INNER ERROR"))
    };

    Ok(HttpResponse::Ok().content_type("video/mp4").body(body.into_bytes()))
}