use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FinanceError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Python error: {0}")]
    Py(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl From<FinanceError> for PyErr {
    fn from(err: FinanceError) -> PyErr {
        match err {
            FinanceError::Py(s) => PyRuntimeError::new_err(s),
            _ => PyRuntimeError::new_err(err.to_string()),
        }
    }
}

pub type FinanceResult<T> = Result<T, FinanceError>;
