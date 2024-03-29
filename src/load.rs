use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};
use tokio::{fs, sync::mpsc};
use std::path;
use crate::{model, error};
use model::Atom;

pub async fn load_xyz(tx0: mpsc::Sender<Atom>, tx1: mpsc::Sender<i64>, path: String) 
-> Result<(), error::SQLMDError> {
    
    let filepath = path::Path::new(&path);
    let file = fs::File::open(filepath).await?;
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
        let atom_count = header.trim().parse::<i64>()?;

        // Load comment line to extract step number
        let line = lines.next_line().await?;
        let comment = match line {
            Some(s) => s,
            None => return Err(error::SQLMDError::NotFoundError(
                format!("Comment line not found in {}", path)
            ))            
        };
        let idx = match &comment.find("iter:") {
            Some(num) => num + 5,
            None => return Err(error::SQLMDError::NotFoundError(
                format!("keyword 'iter:' not found in {}", path)
            ))
        };
        let s = &comment[idx..comment.len()];
        let step = s.split_whitespace().next().unwrap().parse::<i64>().unwrap();
        tx1.send(step).await.unwrap();

        // Load atom parameters
        for i in 0..atom_count {
            let line = lines.next_line().await?;
            let atom_str = match line {
                None => continue,
                Some(s) => s,
            };
            let mut params = atom_str.split_whitespace();
            let atom = Atom{
                step: step,
                atom_id: i,
                element: params.next().unwrap_or("X").to_string(),
                x: params.next().unwrap().parse::<f64>().unwrap(),
                y: params.next().unwrap().parse::<f64>().unwrap(),
                z: params.next().unwrap().parse::<f64>().unwrap(),
                charge: params.next().unwrap().parse::<f64>().unwrap(),
                vx: params.next().unwrap().parse::<f64>().unwrap(),
                vy: params.next().unwrap().parse::<f64>().unwrap(),
                vz: params.next().unwrap().parse::<f64>().unwrap(),
            };
            tx0.send(atom).await.unwrap();
        }
    }

    Ok(())

}


// Print current treated MD step number
pub async fn print_log(mut rx: mpsc::Receiver<i64>, mut stdout: io::Stdout) {
    
    while let Some(step) = rx.recv().await {
        let log = format!("Loading MD step: {}\r", &step);
        let _r = stdout.write(log.as_bytes()).await;
    }

}