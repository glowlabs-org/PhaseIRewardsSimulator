use std::fmt;

#[derive(Debug)]
pub enum PreprocessorError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidInput(String),
    BigIntParse(num_bigint::ParseBigIntError),
}

impl fmt::Display for PreprocessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreprocessorError::Io(err) => write!(f, "IO error: {err}"),
            PreprocessorError::Json(err) => write!(f, "JSON error: {err}"),
            PreprocessorError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            PreprocessorError::BigIntParse(err) => write!(f, "BigInt parse error: {err}"),
        }
    }
}

impl std::error::Error for PreprocessorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PreprocessorError::Io(err) => Some(err),
            PreprocessorError::Json(err) => Some(err),
            PreprocessorError::BigIntParse(err) => Some(err),
            PreprocessorError::InvalidInput(_) => None,
        }
    }
}

impl From<std::io::Error> for PreprocessorError {
    fn from(err: std::io::Error) -> Self {
        PreprocessorError::Io(err)
    }
}

impl From<serde_json::Error> for PreprocessorError {
    fn from(err: serde_json::Error) -> Self {
        PreprocessorError::Json(err)
    }
}

impl From<num_bigint::ParseBigIntError> for PreprocessorError {
    fn from(err: num_bigint::ParseBigIntError) -> Self {
        PreprocessorError::BigIntParse(err)
    }
}
