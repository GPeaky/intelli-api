use std::sync::Arc;

use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};

use crate::entity::{Championship, SharedUser};

// User Management
#[derive(Debug, Deserialize, Validate)]
pub struct UserUpdateData {
    #[serde(default, deserialize_with = "option_string_trim")]
    #[garde(inner(ascii, length(min = 3, max = 20)))]
    pub username: Option<String>,
    #[serde(default, deserialize_with = "option_string_trim")]
    #[garde(inner(ascii, length(min = 10, max = 100)))]
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserInvitationData {
    #[serde(deserialize_with = "string_trim")]
    #[garde(email)]
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct UserProfileData {
    pub user: SharedUser,
    pub championships: Vec<Arc<Championship>>,
}

// Path Parameters
#[derive(Debug, Deserialize, Validate)]
pub struct UserId(#[garde(range(min = 600000000, max = 699999999))] pub i32);
