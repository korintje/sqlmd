use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::fs;
use tokio::sync::mpsc;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{FromRow, SqliteConnection, Connection, Sqlite, Executor};
use std::path;
mod model;
use model::Atom;
use serde::{Serialize};

#[derive(FromRow, Serialize)]
pub struct TableCount {
    pub count: i32,
}

/*
pub async fn is_table_exist(mut conn: &sqlx::SqliteConnection, table_name: &str) -> bool {
    let table_count: TableCount = sqlx::query_as(
        "SELECT COUNT(*) as count FROM sqlite_master WHERE TYPE='table' AND name=$1"
    )
    .bind(table_name)
    .fetch_one(&mut conn)
    .await.unwrap();
    if table_count.count == 0 {false} else {true}
}
*/

pub async fn load_xyz(path: &path::Path, tx: mpsc::Sender<Atom>) -> Result<(), String> {
    
    let file = match fs::File::open(path).await {
      Err(why) => panic!("couldn't open {}: {}", path.display(), why),
      Ok(file) => file,
    };
    let mut lines = BufReader::new(file).lines();
    
    loop {

        // Load header line
        let line = match lines.next_line().await {
            Err(_e) => break,
            Ok(l) => l,
        };
        let header = match line {
            None => break,
            Some(s) => s,
        };
        let atom_count = match header.trim().parse::<i64>() {
            Err(e) => panic!("couldn't convert header to integer.\n {}", e),
            Ok(num) => num,
        };

        // Load comment line to extract step number
        let line = match lines.next_line().await {
            Err(e) => panic!("Failed to read .xyz comment line.\n {}", e),
            Ok(l) => l,
        };
        let comment = match line {
            None => panic!("Could not to find step number in .xyz file."),
            Some(s) => s,
        };
        let idx = &comment.find("iter:").unwrap() + 5;
        let s = &comment[idx..comment.len()];
        let step = s.split_whitespace().next().unwrap().parse::<i64>().unwrap();

        println!("Step: {}", &step);

        // Load atom parameters
        for i in 0..atom_count {
            let line = match lines.next_line().await {
                Err(e) => panic!("Failed to get atom parameters in .xyz.\n {}", e),
                Ok(l) => l,
            };
            let atom_str = match line {
                None => panic!("Could not to find atom parameters in .xyz file."),
                Some(s) => s,
            };
            let mut params = atom_str.split_whitespace();
            let atom = Atom{
                step: step,
                atom_id: i,
                element: params.next().unwrap_or("X").to_string(),
                charge: params.next().unwrap().parse::<f64>().unwrap(),
                x: params.next().unwrap().parse::<f64>().unwrap(),
                y: params.next().unwrap().parse::<f64>().unwrap(),
                z: params.next().unwrap().parse::<f64>().unwrap(),
                vx: params.next().unwrap().parse::<f64>().unwrap(),
                vy: params.next().unwrap().parse::<f64>().unwrap(),
                vz: params.next().unwrap().parse::<f64>().unwrap(),
            };
            tx.send(atom).await.unwrap();
        }
    }

    Ok(())

}

async fn write_db(mut rx: mpsc::Receiver<Atom>, dbpath: &str) {
    
    let _r1 = Sqlite::create_database(dbpath).await;
    // let pool = SqlitePoolOptions::new().max_connections(32).connect(dbpath).await.unwrap();
    let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
    let table_count: TableCount = sqlx::query_as(
            "SELECT COUNT(*) as count FROM sqlite_master WHERE TYPE='table' AND name=$1"
        ).bind("traj").fetch_one(&mut conn).await.unwrap();
        let is_table_exist = if table_count.count == 0 {false} else {true};

    if !is_table_exist {
        println!("not exist");
        let _r2 = &conn.execute(sqlx::query(
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
        )).await;
    }

    let mut query_str = "INSERT INTO traj VALUES ".to_string();
    let mut counter = 0;

    while let Some(atom) = rx.recv().await {         

        if counter < 25000 {
            let values = format!(
                "({}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                atom.step, atom.atom_id,
                atom.element, atom.charge,
                atom.x, atom.y, atom.z,
                atom.vx, atom.vy, atom.vz,
            );
            query_str += &values;
            counter += 1;
        } else {
            let _r2 = &conn.execute(sqlx::query(&query_str)).await;
            query_str = "INSERT INTO traj VALUES ".to_string();
            counter = 0;
        }
        
    }
    let _r2 = &conn.execute(sqlx::query(&query_str)).await;

    
    /*
    while let Some(atom) = rx.recv().await {
        /*
        let _r3 = sqlx::query("INSERT INTO traj VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(atom.step).bind(atom.atom_id)
            .bind(atom.element).bind(atom.charge)
            .bind(atom.x).bind(atom.y).bind(atom.z)
            .bind(atom.vx).bind(atom.vy).bind(atom.vz)
            .execute(&pool).await;
        */
        let _3 = sqlx::
    }
    */

}


#[tokio::main]
async fn main() {
    
    let dbpath = "geo_end.db";
    let xyzpath = "geo_end.xyz";
    let xyzpath = path::Path::new(xyzpath);
    
    let (tx, rx) = mpsc::channel(1024000);
    let handle1 = tokio::spawn(load_xyz(xyzpath, tx));
    let handle2 = tokio::spawn(write_db(rx, dbpath));
    if let (Ok(_res1), Ok(_res2)) = tokio::join!(handle1, handle2) {
        println!("OK");
    }

}