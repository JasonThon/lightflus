#[cfg(feature = "proto-common")]
pub mod common;
#[cfg(feature = "proto-common")]
pub mod common_impl;

#[cfg(feature = "coordinator")]
pub mod coordinator;

#[cfg(feature = "coordinator")]
pub mod coordinator_gateway;

#[cfg(feature = "coordinator")]
pub mod coordinator_impl;

#[cfg(feature = "worker")]
pub mod worker;

#[cfg(feature = "worker")]
pub mod worker_gateway;

#[cfg(feature = "worker")]
pub mod worker_impl;

#[cfg(feature = "apiserver")]
pub mod apiserver;
#[cfg(feature = "apiserver")]
pub mod apiserver_impl;

pub(crate) const DEFAULT_CONNECT_TIMEOUT: u64 = 3;