#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

pub mod api;
pub mod cli;
pub mod commands;
pub mod core;
pub mod database;
pub mod modules;
pub mod services;
pub mod shared;
pub mod utils;
