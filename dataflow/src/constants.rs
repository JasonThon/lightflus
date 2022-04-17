use std::time;

pub(crate) const WATERMARK_ENV: &str = "watermark.firetime";

pub(crate) const DEFAULT_FIRETIME_DURATION: time::Duration = time::Duration::from_millis(100);

pub(crate) const TABLEFLOW_URI_ENV_KEY: &str = "tableflow.data.uri";
