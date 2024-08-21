pub(crate) use auth::*;
pub(crate) use championship::*;
pub(crate) use f1::*;
pub(crate) use server::*;
pub(crate) use templates::*;
pub(crate) use token::*;
pub(crate) use user::*;

// TODO: Restructure the DTOs to be more modular and reusable

mod auth;
mod championship;
mod f1;
mod server;
mod templates;
mod token;
mod user;
