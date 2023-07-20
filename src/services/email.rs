use crate::dtos::RegisterUserDto;
use askama::Template;
use lettre::{
    error::Error,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::str::FromStr;

#[derive(Template)]
#[template(path = "verify.html")]
pub struct VerifyEmailTemplate<'a> {
    pub username: &'a str,
    pub token: &'a str,
}

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
        user: &RegisterUserDto,
        template: VerifyEmailTemplate<'a>,
    ) -> Result<bool, Error> {
        let message = Message::builder()
            .from(self.from_mailbox.to_owned())
            .to(Mailbox::new(
                Some(user.username.clone()),
                Address::from_str(&user.email).unwrap(),
            ))
            .header(ContentType::TEXT_HTML)
            .subject("Hello!, verify your email")
            .body(template.render().unwrap())?;

        Ok(self.mailer.send(message).await.is_ok())
    }
}
