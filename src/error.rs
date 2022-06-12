// use thiserror::Error;
use pyo3::PyErr;
use pyo3::exceptions::{
  PyConnectionError,
  PyIOError,
  PyValueError,
  PyException,
};


#[derive(thiserror::Error, Debug)]
pub enum SQLMDError {

  #[error("SQLite connection failed")]
  SQLError(#[from] sqlx::Error),

  #[error("tokio join failed")]
  JointError(#[from] tokio::task::JoinError),

  #[error("io error")]
  IOError(#[from] std::io::Error),

  #[error("Parse int error")]
  ParseIntError(#[from] std::num::ParseIntError),

  #[error("Not found")]
  NotFoundError(String),

}


impl std::convert::From<SQLMDError> for PyErr {

  fn from(err: SQLMDError) -> PyErr {

    match err {
      SQLMDError::IOError(e) => PyIOError::new_err(e),
      SQLMDError::SQLError(e) => PyConnectionError::new_err(e.to_string()),
      SQLMDError::ParseIntError(e) => PyValueError::new_err(e),
      SQLMDError::JointError(e) => PyException::new_err(e.to_string()),
      SQLMDError::NotFoundError(e) => PyValueError::new_err(e),
    }
  
  }
}