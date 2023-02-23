use rusqlite::{params, Connection, Result, ToSql, OpenFlags};
use std::sync::{Arc, Mutex};
use std::fs;

pub fn create_db_conn() -> Arc<Mutex<Connection>> {
    let database_path = std::env::var("DATABASE").unwrap();
    let conn = match Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE) {
        Ok(conn) => conn,
        Err(_) => {
            // File doesn't exist, create a new one
            fs::File::create(&database_path).unwrap();
            Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap()
        }
    };
    
    let conn = Arc::new(Mutex::new(conn));

   // THAT FUNCTION WILL BE USED IF TABLE NEED TO BE CREATED 
    create_table(&conn.lock().unwrap()).unwrap();
    
    println!("Database connection and table creation successful");
    conn
}

use std::ops::Not;

#[derive(Debug, Clone, Eq, PartialEq )]
pub struct User {
    pub numba: i32,
    pub user_id: String,
    pub address: String,
    pub balance: i32,
}

impl Not for User {
    type Output = bool;

    fn not(self) -> Self::Output {
        self.user_id.is_empty() || self.address.is_empty() || self.balance == 0
    }
}

//  THAT FUNCTION IS USED ONLY ONCE TO CREATE TABLE IF NEEDED
pub fn create_table(conn: &Connection) -> Result<()> {
    // Check if the users table exists
    let table_exists: i32 = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='users')",
        [],
        |row| row.get(0),
    )?;
    if table_exists == 1 {
        println!("Table 'users' already exists");
        return Ok(());
    }

    // Create the users table
    conn.execute(
        "CREATE TABLE users (
            numba       INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id     TEXT NOT NULL UNIQUE,
            address     TEXT,
            balance     INTEGER 
        )",
        params![],
    )?;

    Ok(())
}


pub async fn insert_row(conn: Arc<Mutex<Connection>>, user_id: &str, address: &str, balance: i32) -> Result<()> {
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO users (user_id, address, balance) VALUES (?1, ?2, ?3)",
        params![user_id, address, balance],
    )?;

    Ok(())
}

pub async fn get_user(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str) -> Result<User, rusqlite::Error> {
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT * FROM users WHERE user_id = ?1")?;
    let user_iter = stmt.query_map(params![user_id], |row| {
        Ok(User {
            numba: row.get(0)?,
            user_id: row.get(1)?,
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

pub async fn update_balance(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str, balance: i32) -> Result<()> {
    let conn = conn.lock().unwrap();
    conn.execute("UPDATE users SET balance = ?3 WHERE user_id = ?1", params![balance, user_id])?;
    Ok(())
    // THIS FUNCTION WILL BE USED LATER FOR WEB3 CRATE TO UPDATE USER BALANCE AFTER DEPOSIT
}

pub async fn get_balance(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str) -> Result<i32, rusqlite::Error> {
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT balance FROM users WHERE user_id = ?1")?;
    let balance: i32 = stmt.query_row(params![&user_id], |row| row.get(0))?;
    Ok(balance)
}

pub async fn update_address(conn: &Arc<Mutex<rusqlite::Connection>>, address: &str, user_id: &str) -> Result<()> {
    let conn = conn.lock().unwrap();
    conn.execute("UPDATE users SET address = ?2 WHERE user_id = ?1", params![user_id, address])?;
    Ok(())
}

pub async fn plus_balance(conn: &Arc<Mutex<Connection>>, user_id: &str, amount: i32) -> Result<(), rusqlite::Error> {
    let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE users SET balance = balance + ?1 WHERE user_id = ?2",
        &[&amount as &dyn ToSql, &user_id],
    )?;
    Ok(())
}

pub async fn minus_balance(conn: &Arc<Mutex<rusqlite::Connection>>, user_id: &str, amount: i32) -> Result<(), rusqlite::Error> {
    let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE users SET balance = balance - ?1 WHERE user_id = ?2",
        &[&amount as &dyn ToSql, &user_id],
    )?;
    Ok(())
}

pub async fn add_balance(conn: &Arc<Mutex<Connection>>, user_id: &str) -> Result<(), rusqlite::Error> {
    let balance = get_balance(&conn, user_id).await?;
    if balance < 1 {
        plus_balance(conn, user_id, 5).await?;
    }
    Ok(())
}
