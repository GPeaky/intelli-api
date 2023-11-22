use crate::{
    handlers::{
        admin::pool_status,
        championships::{
            active_sockets, delete_championship, update_championship, user_championships,
        },
        user::{delete_user, disable_user, enable_user},
    },
    middlewares::{Admin, Authentication},
};
use ntex::web;

pub(crate) fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .service(
                web::scope("/users")
                    .route("/{id}", web::delete().to(delete_user))
                    .route("/{id}/enable", web::post().to(enable_user))
                    .route("/{id}/disable", web::post().to(disable_user)),
            )
            .service(
                web::scope("/championships")
                    .route("/{id}", web::get().to(user_championships)) // id = user_id
                    .route("/{id}", web::delete().to(delete_championship))
                    .route("/{id}", web::post().to(update_championship)),
            )
            .service(web::scope("/sockets").route("/sockets", web::get().to(active_sockets)))
            .route("/pools", web::get().to(pool_status))
            .wrap(Admin)
            .wrap(Authentication),
    );
}
