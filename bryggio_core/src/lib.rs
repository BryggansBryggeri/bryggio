#![forbid(unsafe_code)]
#![warn(
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(all(not(test)), warn(clippy::unwrap_used))]

pub mod actor;
pub mod control;
pub(crate) mod hardware;
pub mod sensor;
pub mod time;
pub mod utils;
