use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, FromRequest)]
pub struct CreateGroupForm {
    pub name: String,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupResponse {
    //todo
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRequest)]
pub struct JoinGroup {
    pub password: String,
}