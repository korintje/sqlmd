mod model;
mod load;
mod error;
mod db;
use std::path;
use pyo3::prelude::*;
use tokio::{io, sync::mpsc};
use sha2::{Sha256, Digest};


// Read .db if exists. Read .xyz if .db not exists or .xyz updated
pub async fn read_xyz(filepath: &str) 
-> Result<(), error::SQLMDError> {
    
  let xyzpath = filepath.to_string();
  let dbpath = xyzpath.clone() + ".db";

  if ! path::Path::new(&dbpath).exists() {
    db::create_db(&dbpath).await?;
  }
  let conn = db::connect_db(&dbpath).await?;
  let mut conn = db::prepare_tables(conn).await?;
  
  // Calculate hash of File 'filepath' to detect file updating
  let is_xyz_updated: bool;
  let mut file = std::fs::File::open(&xyzpath)?;
  let mut sha256 = Sha256::new();
  std::io::copy(&mut file, &mut sha256)?;
  let hash_struct = sha256.finalize();
  let xyz_hash = hash_struct.as_slice();
  match db::get_hash(&mut conn).await {
    Ok(hash) => match hash {
      Some(num) => is_xyz_updated = if num == xyz_hash {false} else {true},
      None => is_xyz_updated = true,
    },
    Err(e) => return Err(e),
  }
  match db::save_hash(&mut conn, xyz_hash).await {
    Ok(_) => (),
    Err(e) => return Err(e),
  };

  // If 'filepath' is new or updated, read it and copy to database
  if is_xyz_updated {
    println!("Load trajectory from {}", filepath);
    let stdout = io::stdout();
    let (tx0, rx0) = mpsc::channel(102400);
    let (tx1, rx1) = mpsc::channel(1024);
    
    let load_handle = tokio::spawn(load::load_xyz(tx0, tx1, xyzpath));
    let save_handle = tokio::spawn(db::save_db(rx0, conn));
    let log_handle = tokio::spawn(load::print_log(rx1, stdout));

    let results = tokio::join!(load_handle, save_handle, log_handle);
    match results {
      (Ok(_), Ok(_), Ok(_)) => println!("##### SUCCESSFULLY HANDLED #####"),
      (Err(e), _, _) => return Err(error::SQLMDError::from(e)),
      (_, Err(e), _) => return Err(error::SQLMDError::from(e)),
      (_, _, Err(e)) => return Err(error::SQLMDError::from(e)),
    }
  }else{
    println!("Load trajectory from {}", dbpath);
  }

  Ok(())

}


// Wrapper of read_xyz function for Python.
#[pyfunction]
#[pyo3(name = "read_xyz")]
fn py_read_xyz(filepath: &str) -> PyResult<String> {

  let runtime = tokio::runtime::Builder::new_multi_thread()
              .worker_threads(1).enable_all().build().unwrap();
  match runtime.block_on(read_xyz(filepath)) {
    Ok(_) => Ok("!!! -------- Finished -------- !!!".to_string()),
    Err(e) => Err(pyo3::PyErr::from(e)),
  }

}


// A Python module implemented in Rust.
#[pymodule]
fn sqlmd(_py: Python, m: &PyModule) -> PyResult<()> {
    let _r = m.add_function(wrap_pyfunction!(py_read_xyz, m)?)?;
    Ok(())
}