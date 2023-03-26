use common::err::BizCode;

pub mod api;
pub mod coord;
pub mod executions;
pub mod managers;
pub mod scheduler;
pub mod storage;

pub const COORDINATOR_BIZ_CODE: BizCode = 100;
