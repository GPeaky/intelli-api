use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "verify_email.stpl")]
pub struct VerifyEmail<'a> {
    pub verification_link: &'a str,
}

#[derive(TemplateOnce)]
#[template(path = "email_verified.stpl")]
pub struct EmailVerified {}

#[derive(TemplateOnce)]
#[template(path = "reset_password.stpl")]
pub struct ResetPassword<'a> {
    pub reset_password_link: &'a str,
}

#[derive(TemplateOnce)]
#[template(path = "password_changed.stpl")]
pub struct PasswordChanged {}
