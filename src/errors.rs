use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum SimError {
    Validation(String),
    Algorithm(String),
    Internal(String),
}

impl Display for SimError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::Validation(s) => write!(f, "validation error: {s}"),
            SimError::Algorithm(s) => write!(f, "algorithm error: {s}"),
            SimError::Internal(s) => write!(f, "internal error: {s}"),
        }
    }
}

impl std::error::Error for SimError {}

impl SimError {
    pub fn validation<S: Into<String>>(s: S) -> Self {
        SimError::Validation(s.into())
    }
    pub fn algorithm<S: Into<String>>(s: S) -> Self {
        SimError::Algorithm(s.into())
    }
}
