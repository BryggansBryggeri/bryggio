#![forbid(unsafe_code)]
// Enable clippy if our Cargo.toml file asked us to do so.
// Enable as many useful Rust and Clippy warnings as we can stand.  We'd
// also enable `trivial_casts`, but we're waiting for
// https://github.com/rust-lang/rust/issues/23416.
#![warn(
    //missing_copy_implementations,
    //missing_debug_implementations,
    //missing_docs,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
// Disallow `println!`. Use `debug!` for debug output
// (which is provided by the `log` crate).
// This allows us to use `unwrap` on `Option` values (because doing makes
// working with Regex matches much nicer) and when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test)), warn(clippy::unwrap_used))]

#[warn(clippy::print_stdout)]
#[warn(clippy::cast_possible_truncation)]
#[warn(clippy::cast_possible_wrap)]
#[warn(clippy::cast_precision_loss)]
#[warn(clippy::cast_sign_loss)]
#[warn(clippy::missing_docs_in_private_items)]
#[warn(clippy::mut_mut)]
#[warn(clippy::unseparated_literal_suffix)]
#[warn(clippy::wrong_self_convention)]
pub mod actor;
// pub mod buzzer;
pub mod control;
pub mod data_logger;
mod hardware;
pub mod logger;
pub mod pub_sub;
pub mod sensor;
pub mod supervisor;
mod time;
pub mod utils;
