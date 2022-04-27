use std::time;

pub(crate) const WATERMARK_ENV: &str = "WATERMARK_FIRETIME";

pub(crate) const DEFAULT_FIRETIME_DURATION: time::Duration = time::Duration::from_millis(100);

pub(crate) const TABLEFLOW_URI_ENV_KEY: &str = "TABLEFLOW_DATA_URI";
