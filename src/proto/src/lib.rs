#[cfg(feature = "proto-common")]
#[cfg(not(tarpaulin))]
pub mod common;
#[cfg(feature = "coordinator")]
#[cfg(not(tarpaulin))]
pub mod coordinator;

#[cfg(feature = "worker")]
#[cfg(not(tarpaulin))]
pub mod worker;

#[cfg(feature = "apiserver")]
#[cfg(not(tarpaulin))]
pub mod apiserver;
