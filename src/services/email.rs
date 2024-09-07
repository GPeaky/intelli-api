use std::str::FromStr;

use lettre::{
    message::{header::ContentType, Mailbox, MessageBuilder},
    transport::smtp::{authentication::Credentials, PoolConfig},
    Address, AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use sailfish::TemplateOnce;

use crate::{config::constants::MAX_CONCURRENT_EMAILS, entity::SharedUser, error::AppResult};

/// Asynchronous email service.
#[derive(Clone)]
pub struct EmailService(&'static AsyncSmtpTransport<Tokio1Executor>);

impl EmailService {
    /// Creates a new `EmailService`.
    ///
    /// Configures async SMTP transport using environment variables.
    pub fn new() -> Self {
        let mailer = Box::leak(Box::new(
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
                &dotenvy::var("EMAIL_HOST").unwrap(),
            )
            .unwrap()
            .port(2525)
            .credentials(Credentials::new(
                dotenvy::var("EMAIL_NAME").unwrap(),
                dotenvy::var("EMAIL_PASS").unwrap(),
            ))
            .pool_config(PoolConfig::new().max_size(MAX_CONCURRENT_EMAILS as u32))
            .build(),
        ));

        Self(mailer)
    }

    /// Sends an email.
    ///
    /// # Parameters
    /// - `user`: Email recipient.
    /// - `subject`: Email subject.
    /// - `body`: Email body (must implement `TemplateOnce`).
    ///
    /// # Returns
    /// `AppResult<()>`: Ok if successfully queued, Err otherwise.
    pub async fn send_mail<'a, T>(
        &self,
        user: SharedUser,
        subject: &'a str,
        body: T,
    ) -> AppResult<()>
    where
        T: TemplateOnce,
    {
        let message = MessageBuilder::new()
            .from(Mailbox::new(
                Some(String::from("Intelli Telemetry")),
                Address::from_str(&dotenvy::var("EMAIL_FROM").unwrap()).unwrap(),
            ))
            .to(Mailbox::new(
                Some(user.username.to_owned()),
                Address::from_str(&user.email).unwrap(),
            ))
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.render_once()?)
            .expect("Message builder error");

        let mailer = self.0;

        ntex::rt::spawn(async move {
            if let Err(e) = mailer.send(message).await {
                tracing::error!("Error sending email: {}", e);
            }
        });

        Ok(())
    }
}
