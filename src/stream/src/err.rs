#[derive(Debug, Clone)]
pub struct PipelineError {
    pub kind: ErrorKind,
    pub msg: String,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {}