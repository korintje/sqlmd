use sqlx::{migrate::MigrateDatabase, SqliteConnection, Connection, Sqlite, Executor};
use crate::{model, error};
use error::SQLMDError;

// Create DB file
pub async fn create_db(dbpath: &str) 
-> Result<(), SQLMDError> {
  match Sqlite::create_database(&dbpath).await {
    Ok(_) => Ok(()),
    Err(e) => return Err(SQLMDError::SQLError(e)),
  }  
}

// Connect to DB and return connection
pub async fn connect_db(dbpath: &str)
-> Result<SqliteConnection, SQLMDError> {
  match SqliteConnection::connect(dbpath).await {
    Ok(c) => Ok(c),
    Err(e) => Err(SQLMDError::SQLError(e))
  }
}

// Prepare DB tables
pub async fn prepare_tables(mut conn: SqliteConnection) 
-> Result<SqliteConnection, error::SQLMDError> {
  let table_count: model::TableCount = sqlx::query_as(
    "SELECT COUNT(*) as count FROM sqlite_master WHERE TYPE='table' AND name=$1"
  )
  .bind("traj")
  .fetch_one(&mut conn)
  .await?;
  if table_count.count == 0 {
    if let Err(e) = conn.execute(sqlx::query(
      "CREATE TABLE IF NOT EXISTS traj (
        step        INTEGER NOT NULL,
        atom_id     INTEGER NOT NULL,
        element     TEXT NOT NULL,
        charge      REAL,
        x           REAL,
        y           REAL,
        z           REAL,
        vx          REAL,
        vy          REAL,
        vz          REAL
      )"
    )).await {
      return Err(SQLMDError::SQLError(e))
    };
    if let Err(e) = conn.execute(sqlx::query(
      "CREATE TABLE IF NOT EXISTS metadata (
        id             INTEGER,
        xyzhash        INTEGER 
      )"
    )).await {
      return Err(SQLMDError::SQLError(e))
    };
  }
  Ok(conn)
}


pub async fn get_hash(conn: &mut SqliteConnection) -> Result<Option<u32>, SQLMDError> {
  let hash: Option<(u32,)> = sqlx::query_as(
    "SELECT xyzhash FROM metadata WHERE id = 1"
  ).fetch_optional(conn).await?;
  match hash {
    Some(num) => {
      println!("GOT xyz checksum: {}", num.0);
      Ok(Some(num.0))
    },
    None => Ok(None),
  }
}


pub async fn save_hash(conn: &mut SqliteConnection, hash: u32) -> Result<(), SQLMDError> {
  conn.execute(
    sqlx::query("REPLACE INTO metadata values (1, ?)").bind(hash)
  ).await?;
  Ok(())
}

/*
pub async fn load_db() {

}
*/