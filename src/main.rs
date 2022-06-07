use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};
use tokio::{fs, sync::mpsc};
use sqlx::{migrate::MigrateDatabase, SqliteConnection, Connection, Sqlite, Executor};
use std::path;
mod model;
use model::{Atom, TableCount};


async fn load_xyz(tx0: mpsc::Sender<Atom>, tx1: mpsc::Sender<i64>, path: &path::Path) -> Result<(), String> {
    
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
        tx1.send(step).await.unwrap();

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
            tx0.send(atom).await.unwrap();
        }
    }

    Ok(())

}


async fn save_db(mut rx: mpsc::Receiver<Atom>, dbpath: &str) {

    let _r1 = Sqlite::create_database(dbpath).await;
    let mut conn = SqliteConnection::connect(dbpath).await.unwrap();
    let table_count: TableCount = sqlx::query_as(
            "SELECT COUNT(*) as count FROM sqlite_master WHERE TYPE='table' AND name=$1"
        ).bind("traj").fetch_one(&mut conn).await.unwrap();

    if table_count.count == 0 {
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

    let mut values = vec![];
    let query_head = "INSERT INTO traj VALUES ".to_string();
    let mut counter = 0;

    while let Some(atom) = rx.recv().await {         
        if counter < 5000 {
            let value = format!(
                "({}, {}, '{}', {}, {}, {}, {}, {}, {}, {})",
                atom.step, atom.atom_id,
                atom.element, atom.charge,
                atom.x, atom.y, atom.z,
                atom.vx, atom.vy, atom.vz,
            );
            values.push(value);
            counter += 1;
        } else {
            let query = query_head.clone() + &values.join(", ");
            match &conn.execute(sqlx::query(&query)).await {
                Err(e) => panic!("{}", e),
                Ok(_r) => (),
            };
            values = vec![];
            counter = 0;
        }
    }

    let query = query_head + &values.join(", ");
    match &conn.execute(sqlx::query(&query)).await {
        Err(e) => panic!("{}", e),
        Ok(_r) => (),
    };

}


// Print current treated MD step number
async fn print_log(mut rx: mpsc::Receiver<i64>, mut stdout: io::Stdout) {
    
    while let Some(step) = rx.recv().await {
        let log = format!("Loading MD step: {}\r", &step);
        let _r = stdout.write(log.as_bytes()).await;
    }

}


#[tokio::main]
async fn main() {
    
    let dbpath = "geo_end.db";
    let xyzpath = "geo_end.xyz";
    let xyzpath = path::Path::new(xyzpath);
    
    let stdout = io::stdout();
    let (tx0, rx0) = mpsc::channel(102400);
    let (tx1, rx1) = mpsc::channel(1024);

    let load_handle = tokio::spawn(load_xyz(tx0, tx1, xyzpath));
    let save_handle = tokio::spawn(save_db(rx0, dbpath));
    let _log_handle = tokio::spawn(print_log(rx1, stdout));
    
    if let (Ok(_res1), Ok(_res2)) = tokio::join!(load_handle, save_handle) {
        println!("-------- Finished --------");
    }

}