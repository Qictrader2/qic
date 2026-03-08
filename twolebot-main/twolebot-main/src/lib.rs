#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod claude;
pub mod config;
pub mod cron;
pub mod dispatcher;
pub mod error;
pub mod logging;
pub mod mcp;
pub mod rendering;
pub mod semantic;
pub mod server;
pub mod storage;
pub mod telegram;
pub mod transcription;
pub mod tunnel;
pub mod types;
pub mod work;

pub use config::{Args, Config};
pub use error::{Result, TwolebotError};
