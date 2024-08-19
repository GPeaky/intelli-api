pub(crate) use auth::*;
pub(crate) use championship::*;
pub(crate) use email::*;
pub(crate) use f1::*;
pub(crate) use server::*;
pub(crate) use token::*;
pub(crate) use user::*;

// TODO: Restructure the DTOs to be more modular and reusable

mod auth;
mod championship;
mod email;
mod f1;
mod server;
mod token;
mod user;
