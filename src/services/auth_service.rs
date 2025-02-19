use actix_jwt_auth_middleware::{FromRequest};
use serde::{Deserialize, Serialize};

#[derive(FromRequest, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub id: i64,
    pub role: Role,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Role {
    Admin,
    RegisteredUser,
}
