use rusqlite::{params, Connection, Result};
use std::path::Path;
mod model;
use model::{Atom, Trajectory};

fn create_table(conn: &rusqlite::Connection) -> Result<usize> {
    conn.execute(
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
        )",
        [],
    )
}

fn insert_atom(conn: &rusqlite::Connection, atom: &Atom) -> Result<usize> {
    conn.execute(
        "INSERT INTO traj values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![atom.step, atom.atom_id, atom.element, atom.charge,
            atom.x, atom.y, atom.z, atom.vx, atom.vy, atom.vz
        ]
    )
}

fn main() -> Result<()> {

    let path = "geo_end.db";
    let conn = Connection::open(&path)?;

    create_table(&conn)?;

    let xyzpath = Path::new("geo_end.xyz");
    let traj = Trajectory::from_xyz(&xyzpath);
    println!("{:?}", &traj.atoms);
    for atom in traj.atoms.iter() {
        insert_atom(&conn, &atom)?;
    }  

    let _r = conn.close();

    Ok(())

}
