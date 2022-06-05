use std::io::{BufRead, BufReader};
use std::path::Path;
use std::fs::File;

#[derive(Debug)]
pub struct Atom{
  pub step: i64,
  pub atom_id: i64,
  pub element:String,
  pub charge: f64,
  pub x: f64,
  pub y: f64,
  pub z: f64,
  pub vx: f64,
  pub vy: f64,
  pub vz: f64,
}

pub struct Trajectory {
  pub atoms: Vec<Atom>,
}


impl Trajectory {

  pub fn from_xyz(path: &Path) -> Trajectory {

    let mut atoms: Vec<Atom> = vec![];
    
    let file = match File::open(path) {
      Err(why) => panic!("couldn't open {}: {}", path.display(), why),
      Ok(file) => file,
    };
    
    let reader = BufReader::new(file);
    let mut iterator = reader.lines();

    loop {
      let atom_count: i64;
      match iterator.next() {
        None => break,
        Some(item) => {
          let s = item.unwrap();
          atom_count = match s.trim().parse::<i64>() {
            Ok(i) => i,
            Err(_e) => {
              panic!("couldn't convert header {} to integer.", s);
            }
          };
        },
      }
      let mut step: i64 = 0;
      if let Some(item) = iterator.next() {
        let comment = item.unwrap();
        let idx = &comment.find("iter:").unwrap() + 5;
        let s = &comment[idx..comment.len()];
        step = s.split_whitespace().next().unwrap().parse::<i64>().unwrap();
      }
      for i in 0..atom_count {
        let line = iterator.next().unwrap().unwrap();
        let mut params = line.split_whitespace();
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
        atoms.push(atom);
      }
    }

    Trajectory{atoms}
  }

}