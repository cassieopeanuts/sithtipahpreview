use rusqlite::{params, Connection, Result};
pub struct User {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub balance: i32,
}

pub fn main() -> Result<()> {
    let conn = Connection::open("userstable.db")?;

    create_table(&conn);

    Ok(())
}
pub async fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE users (
                  id              INTEGER,
                  name            TEXT NOT NULL,
                  address         TEXT,
                  balance         INTEGER NOT NULL,
                  PRIMARY KEY (id)
                  )",
        params![],
    )?;

    Ok(())
}

pub async fn insert_row(conn: &Connection, name: &str, address: &str, balance: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO users (name, address, balance) VALUES (?1, ?2, ?3)",
        params![name, address, balance],
    )?;

    Ok(())
}

pub async fn get_user(conn: &rusqlite::Connection, user_id: u64) -> Result<User, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM users WHERE user_id = ?1")?;
    let user_iter = stmt.query_map(params![user_id], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            address: row.get(2)?,
            balance: row.get(3)?,
        })
    })?;

    let mut user = None;
    for u in user_iter {
        user = Some(u?);
        break;
    }

    match user {
        Some(u) => Ok(u),
        None => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

pub async fn update_balance(conn: &Connection, user_id: u64, new_balance: i32) -> Result<()> {
    conn.execute("UPDATE users SET balance = ?1 WHERE id = ?2", params![new_balance, user_id])?;
    Ok(())
}

pub async fn get_balance(conn: &Connection, user_id: u64) -> Result<i32, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT balance FROM users WHERE id = ?1")?;
    let balance: i32 = stmt.query_row(params![user_id], |row| row.get(0))?;
    Ok(balance)
}

pub async fn update_address(conn: &Connection, id: u64, address: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE users SET address = ?1 WHERE id = ?2",
        params![address, id],
    );
    Ok(())
}