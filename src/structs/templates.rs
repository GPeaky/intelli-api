use sailfish::TemplateOnce;

// Email Templates
#[derive(TemplateOnce)]
#[template(path = "verify_email.stpl")]
pub struct EmailVerificationTemplate<'a> {
    pub verification_link: &'a str,
}

#[derive(TemplateOnce)]
#[template(path = "email_verified.stpl")]
pub struct EmailVerificationConfirmationTemplate {}

#[derive(TemplateOnce)]
#[template(path = "reset_password.stpl")]
pub struct PasswordResetTemplate<'a> {
    pub reset_password_link: &'a str,
}

#[derive(TemplateOnce)]
#[template(path = "password_changed.stpl")]
pub struct PasswordChangeConfirmationTemplate {}
