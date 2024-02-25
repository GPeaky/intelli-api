use std::str::FromStr;

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use loole::Sender;
use once_cell::sync::Lazy;
use sailfish::TemplateOnce;
use tracing::error;

use crate::{
    error::{AppResult, CommonError},
    structs::EmailUser,
};

static MAILBOX: Lazy<Mailbox> = Lazy::new(|| {
    Mailbox::new(
        Some("Intelli Telemetry".to_owned()),
        Address::from_str(dotenvy::var("EMAIL_FROM").as_ref().unwrap()).unwrap(),
    )
});

/// A service for sending emails asynchronously.
///
/// This struct encapsulates the functionality to send emails using an asynchronous SMTP transport.
/// It leverages a bounded channel for message queuing and delivery, ensuring that email sending
/// operations do not block the main execution thread. The service is designed to handle
/// potentially high volumes of email sending tasks with resilience.
#[derive(Clone)]
pub struct EmailService(Sender<Message>);

// Todo: Implement a pool of receivers to send emails in case of a single receiver can't handle the load
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
        let (tx, rx) = loole::bounded(50);

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

        Self(tx)
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
    /// the `TemplateOnce` trait, allowing for dynamic content generation.
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
    pub fn send_mail<'a, T: TemplateOnce>(
        &self,
        user: EmailUser<'a>,
        subject: &'a str,
        body: T,
    ) -> AppResult<()> {
        let message = Message::builder()
            .from(unsafe { std::ptr::read(&*MAILBOX as *const Mailbox) })
            .to(Mailbox::new(
                Some(user.username.to_string()),
                Address::from_str(user.email).unwrap(),
            ))
            .header(ContentType::TEXT_HTML)
            .subject(subject)
            .body(body.render_once()?)
            .expect("Message builder error");

        self.0.send(message).map_err(|e| {
            error!("Error sending email: {}", e);
            CommonError::SendMail
        })?;

        Ok(())
    }
}
