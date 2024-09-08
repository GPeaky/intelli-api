use ntex::web::{self, delete, get, post, scope, ServiceConfig};

use crate::{
    handlers::{admin::server_active_pools, championships, user},
    middlewares::{Admin, Authentication},
};

pub(crate) fn admin_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .wrap(Admin)
            .wrap(Authentication)
            .service(
                scope("/users/{id}")
                    .route("", delete().to(user::admin::remove_user))
                    .route("/championships", get().to(user::admin::user_championships))
                    .route(
                        "/reactivate",
                        post().to(user::admin::reactivate_user_account),
                    )
                    .route(
                        "/deactivate",
                        post().to(user::admin::deactivate_user_account),
                    ),
            )
            .service(scope("/championships").route(
                "/{id}",
                delete().to(championships::admin::delete_championship),
            ))
            .route(
                "/services",
                get().to(championships::admin::active_championships),
            )
            .route("/pools", get().to(server_active_pools)),
    );
}
