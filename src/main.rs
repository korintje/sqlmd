mod model;
mod load;
mod error;
mod db;
use std::{path, env};
use tokio::{io, sync::mpsc};
use sha2::{Sha256, Digest};


// Read .db if exists. Read .xyz if .db not exists or .xyz updated
#[tokio::main]
async fn main() {

  // Set filepath
  let xyzpath: &str;
  let args: Vec<String> = env::args().collect();
  let len_args = args.len();
  if len_args <= 1 {
    xyzpath = "geo_end.xyz";
  } else {
    xyzpath = &args[1];
  }
  let dbpath = xyzpath.to_string() + ".db";
  println!("source xyz filepath: {}", &xyzpath);

  // Create SQLite database
  let db_exist: bool;
  if path::Path::new(&dbpath).exists() {
    println!("database file {} found", &dbpath);
    db_exist = true;
  } else {
    println!("creating database file {}", &dbpath);
    db::create_db(&dbpath).await.unwrap();
    db_exist = false;
  }
  let conn = db::connect_db(&dbpath).await.unwrap();
  let mut conn = db::prepare_tables(conn).await.unwrap();
  println!("connected to database: {}", &dbpath);
  
  // Calculate hash of 'filepath' to detect file update
  let is_xyz_updated: bool;
  if db_exist == false {
    is_xyz_updated = true;
  } else {
    println!("calculating hash of {} ...", &xyzpath);
    let mut file = std::fs::File::open(&xyzpath).unwrap();
    let mut sha256 = Sha256::new();
    std::io::copy(&mut file, &mut sha256).unwrap();
    let hash_struct = sha256.finalize();
    let xyz_hash = hash_struct.as_slice();
    match db::get_hash(&mut conn).await {
      Ok(hash) => match hash {
        Some(num) => is_xyz_updated = if num == xyz_hash {false} else {true},
        None => is_xyz_updated = true,
      },
      Err(e) => panic!("{}: {}", e, &dbpath),
    }
    match db::save_hash(&mut conn, xyz_hash).await {
      Ok(_) => (),
      Err(e) => panic!("{}: {}", e, &dbpath),
    };
    println!("Finish hash calculation");
  }

  // If 'filepath' is new or updated, read it and copy to database
  if is_xyz_updated {
    println!("Load trajectory from {}", &xyzpath);
    let stdout = io::stdout();
    let (tx0, rx0) = mpsc::channel(102400);
    let (tx1, rx1) = mpsc::channel(1024);
    
    let load_handle = tokio::spawn(load::load_xyz(tx0, tx1, xyzpath.to_string()));
    let save_handle = tokio::spawn(db::save_db(rx0, conn));
    let _log_handle = tokio::spawn(load::print_log(rx1, stdout));
    if let (Ok(_res1), Ok(_res2)) = tokio::join!(load_handle, save_handle) {
      println!("-------- Finished --------");
    }
  } else {
    println!("Load trajectory from {}", dbpath);
  }

}
