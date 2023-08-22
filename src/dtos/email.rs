use super::RegisterUserDto;
use askama::Template;

#[derive(Debug)]
pub struct EmailUser {
    pub username: String,
    pub email: String,
}

impl From<RegisterUserDto> for EmailUser {
    fn from(user: RegisterUserDto) -> Self {
        Self {
            username: user.username,
            email: user.email,
        }
    }
}

pub enum Templates<'a> {
    VerifyEmail(VerifyEmailTemplate<'a>),
    ResetPassword(ResetPasswordTemplate<'a>),
}

#[derive(Template)]
#[template(path = "mail/verify.html")]
pub struct VerifyEmailTemplate<'a> {
    pub username: &'a str,
    pub token: &'a str,
}

#[derive(Template)]
#[template(path = "mail/forgot.html")]
pub struct ResetPasswordTemplate<'a> {
    pub name: &'a str,
    pub token: &'a str,
}
