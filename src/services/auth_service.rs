use std::future::Future;
use std::pin::Pin;
use actix_web::{ FromRequest, HttpMessage, HttpRequest};
use actix_web::body::MessageBody;
use actix_web::dev::Payload;
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::Algorithm::HS256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserClaims {
    pub id: i64,
    pub role: Role,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Role {
    Admin,
    RegisteredUser,
}

impl FromRequest for UserClaims {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        let cookie = req.headers().get("Cookie").unwrap().to_str().unwrap().to_owned();
        let access_token = cookie.split(';').nth(0).unwrap()
            .split('=').nth(1).unwrap().to_owned();

        Box::pin( async move {
            get_claim(&access_token).await
        })
    }
}

pub async fn get_claim(token: &String) -> Result<UserClaims, actix_web::Error> {
    dotenv::dotenv().ok();
    let decoded_token = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_PRIVATE_KEY").unwrap().to_string().as_ref()),
        &Validation::new(HS256)
    );

    match decoded_token {
        Ok(token_data) => Ok(token_data.claims),
        Err(_e) => Err(actix_web::error::ErrorUnauthorized("Unauthorized")),
    }
}