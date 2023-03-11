pub mod collections;
#[cfg(not(tarpaulin_include))]
pub mod consts;
pub mod db;
pub mod err;
pub mod event;
pub mod kafka;
pub mod net;
pub mod redis;
pub mod types;
pub mod utils;
pub mod testutils;

pub const NANOS_PER_MILLI: i64 = 1_000_000;
