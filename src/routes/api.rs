use ntex::web::{delete, get, post, put, resource, scope, ServiceConfig};

use crate::{
    handlers::{
        auth::{
            callback, forgot_password, login, logout, refresh_token, register, reset_password,
            verify_email,
        },
        championships::{
            add_user, all_championships, create_championship, get_championship, remove_user,
            service_status, session_socket, start_service, stop_service, update,
        },
        heartbeat,
        user::{update_user, user_data},
    },
    middlewares::Authentication,
};

#[inline(always)]
pub(crate) fn api_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/auth")
            .route("/google/callback", get().to(callback))
            .route("/register", post().to(register))
            .route("/login", post().to(login))
            .route("/refresh", get().to(refresh_token))
            .route("/verify/email", get().to(verify_email))
            .route("/forgot-password", post().to(forgot_password))
            .route("/reset-password", post().to(reset_password)),
    );

    cfg.service(
        resource("/logout")
            .route(get().to(logout))
            .wrap(Authentication),
    );

    cfg.service(
        scope("/user")
            .route("", put().to(update_user))
            .route("/data", get().to(user_data))
            .wrap(Authentication),
    );

    cfg.service(
        scope("/championships")
            .route("", post().to(create_championship))
            .route("/all", get().to(all_championships))
            .route("/{id}", get().to(get_championship))
            .route("/{id}", put().to(update))
            .route("/{id}/user/add", put().to(add_user))
            .route("/{id}/user/{user_id}", delete().to(remove_user))
            .route("/{id}/service/start", get().to(start_service))
            .route("/{id}/service/status", get().to(service_status))
            .route("/{id}/service/stop", get().to(stop_service))
            .wrap(Authentication),
    );

    cfg.route("/heartbeat", get().to(heartbeat));

    cfg.route("/web_socket/championship/{id}", get().to(session_socket));
}
