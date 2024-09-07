use std::str::FromStr;

use lettre::{
    message::{header::ContentType, Mailbox, MessageBuilder},
    transport::smtp::{authentication::Credentials, PoolConfig},
    Address, AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use sailfish::TemplateOnce;
use tracing::error;

use crate::{config::constants::MAX_CONCURRENT_EMAILS, entity::SharedUser, error::AppResult};

/// A service for sending emails asynchronously.
///
/// This struct encapsulates the functionality to send emails using an asynchronous SMTP transport.
/// It leverages a bounded channel for message queuing and delivery, ensuring that email sending
/// operations do not block the main execution thread. The service is designed to handle
/// potentially high volumes of email sending tasks with resilience.
#[derive(Clone)]
pub struct EmailService(&'static AsyncSmtpTransport<Tokio1Executor>);

impl EmailService {
    /// Constructs a new `EmailService`.
    ///
    /// Initializes the email service with a bounded sender-receiver pair and a default mailbox.
    /// The sender part of the channel is used to enqueue emails for sending, while a separate
    /// Tokio task is spawned to handle sending these emails asynchronously using an SMTP relay.
    ///
    /// The SMTP transport configuration, including the relay host, port, and credentials, are
    /// read from environment variables.
    ///
    /// # Examples
    ///
    /// ```
    /// let email_svc = EmailService::new();
    /// ```
    pub fn new() -> Self {
        let mailer = Box::leak(Box::new(
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
                dotenvy::var("EMAIL_HOST").unwrap().as_str(),
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

    /// Sends an email to a specified recipient.
    ///
    /// Constructs an email message using the provided user information, subject, and body,
    /// then enqueues this message for sending. The body of the email is generated from a template.
    ///
    /// # Parameters
    /// - `user`: The recipient of the email. Contains username and email address.
    /// - `subject`: The subject line of the email.
    /// - `body`: The body of the email. This parameter is expected to be a type that implements
    ///     the `TemplateOnce` trait, allowing for dynamic content generation.
    ///
    /// # Returns
    /// An `AppResult<()>` indicating the outcome of the operation. On success, it returns `Ok(())`.
    /// On failure, it returns an error encapsulating the issue encountered during execution.
    ///
    /// # Examples
    ///
    /// ```
    /// let result = email_svc.send_mail(user, "Welcome!", template);
    /// if result.is_ok() {
    ///     println!("Email sent successfully");
    /// } else {
    ///     println!("Failed to send email");
    /// }
    /// ```
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
                error!("Error sending email: {}", e);
            }
        });

        Ok(())
    }
}
