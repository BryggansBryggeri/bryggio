#![forbid(unsafe_code)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
mod actor;
pub mod api;
pub mod brewery;
pub mod config;
mod control;
mod hardware;
pub mod sensor;
mod utils;
