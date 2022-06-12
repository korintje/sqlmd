// use thiserror::Error;
use pyo3::PyErr;
use pyo3::exceptions::{PyConnectionError};


#[derive(thiserror::Error, Debug)]
pub enum SQLMDError {

  #[error("SQLite connection failed")]
  ConnectionFailed(#[from] sqlx::Error),

  #[error("tokio join failed")]
  JoinFailed(String),

}

impl std::convert::From<tokio::task::JoinError> for SQLMDError {

  fn from(err: tokio::task::JoinError) -> SQLMDError {
    SQLMDError::JoinFailed(err.to_string())
  }

}



impl std::convert::From<SQLMDError> for PyErr {

  fn from(err: SQLMDError) -> PyErr {
    PyConnectionError::new_err(err.to_string())
  
  }
}