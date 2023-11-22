use super::RegisterUserDto;
use askama::Template;

#[derive(Template)]
#[template(path = "verify_email.html")]
pub struct VerifyEmail<'a> {
    pub verification_link: &'a str,
}

#[derive(Template)]
#[template(path = "reset_password.html")]
pub struct ResetPassword<'a> {
    pub reset_password_link: &'a str,
}

#[derive(Template)]
#[template(path = "password_changed.html")]
pub struct PasswordChanged {}

#[derive(Debug)]
pub struct EmailUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
}

impl<'a> From<&'a RegisterUserDto> for EmailUser<'a> {
    fn from(user: &'a RegisterUserDto) -> Self {
        Self {
            username: &user.username,
            email: &user.email,
        }
    }
}
