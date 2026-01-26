//! OAuth endpoint handlers.

#![cfg(feature = "axum")]

mod authorize;
mod metadata;
mod register;
mod revoke;
mod token;

pub use authorize::authorize_handler;
pub use metadata::metadata_handler;
pub use register::register_handler;
pub use revoke::revoke_handler;
pub use token::token_handler;
