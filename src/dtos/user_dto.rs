use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRequest)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRequest)]
pub struct UserRegister {
    pub username: String,
    pub email: String,
    pub password: String,
}