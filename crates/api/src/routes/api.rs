use std::net::IpAddr;

use dashmap::DashMap;
use ntex::web::{self, delete, get, post, put, resource, scope, ServiceConfig};

use crate::{
    handlers::{auth, championships, system_health_check, user},
    middlewares::{Authentication, LoginLimit, VisitorData},
};

#[inline]
pub(crate) fn api_routes(cfg: &mut ServiceConfig, visitors: &'static DashMap<IpAddr, VisitorData>) {
    cfg.service(
        scope("/auth")
            .route("/register", post().to(auth::register))
            .service(
                resource("/login")
                    .route(post().to(auth::login))
                    .wrap(LoginLimit::new(visitors)),
            )
            .service(
                resource("/logout")
                    .route(get().to(auth::logout))
                    .wrap(Authentication),
            )
            .route("/refresh", get().to(auth::refresh_token))
            .route("/verify/email", get().to(auth::verify_email))
            .service(
                scope("/password")
                    .route("/forgot", post().to(auth::forgot_password))
                    .route("/reset", post().to(auth::reset_password)),
            )
            .route("/discord/callback", get().to(auth::discord_callback)),
    );

    cfg.service(
        scope("/user")
            .route("", get().to(user::get))
            .route("", put().to(user::update))
            .route("/championships", get().to(user::get_championships))
            .wrap(Authentication),
    );

    cfg.service(
        scope("/championships")
            .route("", post().to(championships::core::create))
            .service(
                scope("/{id}")
                    .route("", get().to(championships::core::get))
                    .route("", put().to(championships::core::update))
                    .service(
                        scope("/users")
                            .route("", put().to(championships::core::add_user))
                            .route("/{user_id}", delete().to(championships::core::remove_user)),
                    ),
            )
            .wrap(Authentication),
    );

    cfg.service(
        scope("/services")
            .service(
                scope("/championships/{id}")
                    .route("/start", post().to(championships::service::start))
                    .route("/status", get().to(championships::service::status))
                    .route("/stop", post().to(championships::service::stop)),
            )
            .wrap(Authentication),
    );

    cfg.service(scope("/system").route("/health-check", get().to(system_health_check)));

    cfg.service(
        scope("/stream").service(
            scope("/championships/{championship_id}")
                .route("", get().to(championships::stream::stream_live_session))
                .service(
                    web::resource("/telemetry")
                        .wrap(Authentication)
                        .route(get().to(championships::stream::stream_telemetry_session)),
                ),
        ),
    );
}
