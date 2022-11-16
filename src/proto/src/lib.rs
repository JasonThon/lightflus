#[cfg(feature = "proto-common")]
#[cfg(not(tarpaulin_include))]
pub mod common;
#[cfg(feature = "coordinator")]
#[cfg(not(tarpaulin_include))]
pub mod coordinator;

#[cfg(feature = "worker")]
#[cfg(not(tarpaulin_include))]
pub mod worker;

#[cfg(feature = "apiserver")]
#[cfg(not(tarpaulin_include))]
pub mod apiserver;
