use crate::dtos::{EmailUser, Templates};
use askama::Template;
use lettre::{
    error::Error,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::str::FromStr;

#[derive(Clone)]
pub struct EmailService {
    from_mailbox: Mailbox,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    pub fn new() -> Self {
        Self {
            from_mailbox: Mailbox::new(
                Some("Intelli Telemetry".to_owned()),
                Address::from_str(dotenvy::var("EMAIL_NAME").as_ref().unwrap()).unwrap(),
            ),
            mailer: AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
                dotenvy::var("EMAIL_HOST").unwrap().as_str(),
            )
            .unwrap()
            .port(587)
            .credentials(Credentials::new(
                dotenvy::var("EMAIL_NAME").unwrap(),
                dotenvy::var("EMAIL_PASS").unwrap(),
            ))
            .build(),
        }
    }

    pub async fn send_mail<'a>(
        &self,
        user: &EmailUser,
        template: Templates<'a>,
    ) -> Result<bool, Error> {
        let (body, subject) = match template {
            Templates::VerifyEmail(template) => (template.render().unwrap(), "Verify Email"),
            Templates::ResetPassword(template) => (template.render().unwrap(), "Reset Password"),
        };

        let message = Message::builder()
            .from(self.from_mailbox.to_owned())
            .to(Mailbox::new(
                Some(user.username.clone()),
                Address::from_str(&user.email).unwrap(),
            ))
            .header(ContentType::TEXT_HTML)
            .subject(subject)
            .body(body)?;

        Ok(self.mailer.send(message).await.is_ok())
    }
}
