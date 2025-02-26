use std::future::Future;
use std::pin::Pin;
use actix_web::{error, Error, FromRequest, HttpRequest, HttpResponse, ResponseError};
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Payload, ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::Algorithm::HS256;
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;

#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct UserClaims {
    pub id: i64,
    pub role: Role,
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Role {
    Admin,
    RegisteredUser,
}

#[derive(Debug)]
pub struct CookieError {
    message: String,
}

impl CookieError {
    fn new(message: &str) -> CookieError {
        CookieError {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for CookieError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for CookieError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::Unauthorized().body(self.message.clone())
    }
}

pub fn extract_access_token(req: &HttpRequest) -> Result<String, CookieError> {
    let cookie = req.headers()
        .get("Cookie")
        .ok_or(CookieError::new("Cookie header not found!"))?
        .to_str()
        .map_err(|_| CookieError::new("Cookie header contains invalid characters!"))?
        .to_owned();

    let access_token = cookie.split(';')
        .nth(0)
        .ok_or(CookieError::new("No cookie found in header!"))?
        .split("=")
        .nth(1)
        .ok_or(CookieError::new("Access token not found in cookie!"))?
        .to_owned();

    Ok(access_token)
}

impl FromRequest for UserClaims {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        let access_token = match extract_access_token(&req) {
            Ok(token) => token,
            Err(error) => { return Box::pin(async {
                    Err(error.into())
                });
            }
        };

        Box::pin( async move {
            get_claim(&access_token).await
        })
    }
}

pub async fn get_claim(token: &String) -> Result<UserClaims, actix_web::Error> {
    let decoded_token = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_PRIVATE_KEY").unwrap_or_default().to_string().as_ref()),
        &Validation::new(HS256)
    );

    match decoded_token {
        Ok(token_data) => Ok(token_data.claims),
        Err(_e) => Err(actix_web::error::ErrorUnauthorized("Invalid or expired access token!")),
    }
}

pub async fn is_admin(
    user_claims: UserClaims,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    if user_claims.role != Role::Admin {
        return Err(error::ErrorForbidden("Requires Administrator privileges!"))
    };

    next.call(req).await
}

pub async fn is_registered(
    user_claims: UserClaims,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    if user_claims.role != Role::RegisteredUser {
        return Err(error::ErrorForbidden("Requires Registration!"))
    };

    next.call(req).await
}