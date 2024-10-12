use sailfish::TemplateSimple;

// Email Templates
#[derive(TemplateSimple)]
#[template(path = "verify_email.stpl")]
pub struct EmailVerificationTemplate {
    pub verification_link: String,
}

#[derive(TemplateSimple)]
#[template(path = "email_verified.stpl")]
pub struct EmailVerificationConfirmationTemplate {}

#[derive(TemplateSimple)]
#[template(path = "reset_password.stpl")]
pub struct PasswordResetTemplate {
    pub reset_password_link: String,
}

#[derive(TemplateSimple)]
#[template(path = "password_changed.stpl")]
pub struct PasswordChangeConfirmationTemplate {}
