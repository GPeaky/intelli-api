use std::str::FromStr;

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use loole::Sender;
use sailfish::TemplateOnce;
use tracing::error;

use crate::{error::AppResult, structs::EmailUser};

#[derive(Clone)]
pub struct EmailService(Sender<Message>, Mailbox);

// Todo: Implement a pool of receivers to send emails in case of a single receiver can't handle the load
impl EmailService {
    pub fn new() -> Self {
        let (tx, rx) = loole::bounded(50);

        let mailbox = Mailbox::new(
            Some("Intelli Telemetry".to_owned()),
            Address::from_str(dotenvy::var("EMAIL_FROM").as_ref().unwrap()).unwrap(),
        );

        tokio::spawn(async move {
            let mailer: AsyncSmtpTransport<Tokio1Executor> =
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
                    dotenvy::var("EMAIL_HOST").unwrap().as_str(),
                )
                .unwrap()
                .port(2525)
                .credentials(Credentials::new(
                    dotenvy::var("EMAIL_NAME").unwrap(),
                    dotenvy::var("EMAIL_PASS").unwrap(),
                ))
                .build();

            while let Ok(message) = rx.recv_async().await {
                if let Err(e) = mailer.send(message).await {
                    error!("Error sending email: {}", e);
                }
            }
        });

        Self(tx, mailbox)
    }

    pub fn send_mail<'a, T: TemplateOnce>(
        &self,
        user: EmailUser<'a>,
        subject: &'a str,
        body: T,
    ) -> AppResult<()> {
        let message = Message::builder()
            .from(self.1.to_owned())
            .to(Mailbox::new(
                Some(user.username.to_string()),
                Address::from_str(user.email).unwrap(),
            ))
            .header(ContentType::TEXT_HTML)
            .subject(subject)
            .body(body.render_once()?)
            .expect("Message builder error");

        self.0.send(message)?;

        Ok(())
    }
}
