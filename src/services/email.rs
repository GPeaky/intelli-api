use std::str::FromStr;

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use sailfish::TemplateOnce;

use crate::{error::AppResult, structs::EmailUser};

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
                Address::from_str(dotenvy::var("EMAIL_FROM").as_ref().unwrap()).unwrap(),
            ),
            mailer: AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
                dotenvy::var("EMAIL_HOST").unwrap().as_str(),
            )
            .unwrap()
            .port(2525)
            .credentials(Credentials::new(
                dotenvy::var("EMAIL_NAME").unwrap(),
                dotenvy::var("EMAIL_PASS").unwrap(),
            ))
            .build(),
        }
    }

    pub async fn send_mail<'a, T: TemplateOnce>(
        &self,
        user: EmailUser<'a>,
        subject: &'a str,
        body: T,
    ) -> AppResult<()> {
        let message = Message::builder()
            .from(self.from_mailbox.to_owned())
            .to(Mailbox::new(
                Some(user.username.to_string()),
                Address::from_str(user.email).unwrap(),
            ))
            .header(ContentType::TEXT_HTML)
            .subject(subject)
            .body(body.render_once()?)
            .expect("Message builder error");

        self.mailer.send(message).await?;
        Ok(())
    }
}
