use rusqlite::{params, Connection, Result, ToSql};
use std::sync::{Arc, Mutex};

pub fn create_db_conn() -> Arc<Mutex<Connection>> {
    let conn = Connection::open_in_memory().unwrap();
    let conn = Arc::new(Mutex::new(conn));
    create_table(&conn.lock().unwrap()).unwrap();
    conn
}
use std::ops::Not;

#[derive(Debug, Clone, Eq, PartialEq )]
pub struct User {
    pub user_id: String,
    pub address: String,
    pub balance: i64,
}

impl Not for User {
    type Output = bool;

    fn not(self) -> Self::Output {
        self.user_id.is_empty() || self.address.is_empty()
    }
}

pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE users (
                  user_id       TEXT NOT NULL UNIQUE,
                  address       TEXT,
                  balance       INTEGER NOT NULL,
                  PRIMARY KEY (user_id)
                  )",
        params![],
    )?;

    Ok(())
}

pub async fn insert_row(conn: Arc<Mutex<Connection>>, user_id: &str, address: &str, balance: i32) -> Result<()> {
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO users (user_id, address, balance) VALUES (?0, ?1, ?2)",
        params![user_id, address, balance],
    )?;

    Ok(())
}

pub async fn get_user(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str) -> Result<User, rusqlite::Error> {
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT * FROM users WHERE user_id = ?0")?;
    let user_iter = stmt.query_map(params![user_id], |row| {
        Ok(User {
            user_id: row.get(0)?,
            address: row.get(1)?,
            balance: row.get(2)?,
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

pub async fn update_balance(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str, balance: i32) -> Result<()> {
    let conn = conn.lock().unwrap();
    conn.execute("UPDATE users SET balance = ?2 WHERE user_id = ?0", params![balance, user_id])?;
    Ok(())
}

pub async fn get_balance(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str) -> Result<i32, rusqlite::Error> {
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT balance FROM users WHERE user_id = ?0")?;
    let balance: i32 = stmt.query_row(params![&user_id], |row| row.get(0))?;
    Ok(balance)
}

pub async fn update_address(conn: &Arc<Mutex<rusqlite::Connection>>, address: &str, user_id: &str) -> Result<()> {
    let conn = conn.lock().unwrap();
    conn.execute("UPDATE users SET address = ?1 WHERE user_id = ?0", params![address, user_id])?;
    Ok(())
}

pub async fn plus_balance(conn: &Arc<Mutex<Connection>>, user_id: &str, amount: i32) -> Result<(), rusqlite::Error> {
    let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE users SET balance = balance + 2? WHERE user_id = 0?",
        &[&amount as &dyn ToSql, &user_id],
    );
    Ok(())
}

pub async fn minus_balance(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str, amount: i32) -> Result<(), rusqlite::Error> {
    let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE users SET balance = balance - 2? WHERE user_id = 0?",
        &[&amount as &dyn ToSql, &user_id],
    );
    Ok(())
}