use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct RegisterUserDto {
    pub username: String,
    pub password: String,
    pub email: String,
}
