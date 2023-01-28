use serenity::{
    framework::standard::{ StandardFramework,
        Args, CommandResult,
        macros::{command, group}, Command,
    },
    model::{channel::Message,}
};
use serenity::prelude::Context;
use super::database::{insert_row, get_user, update_balance, get_balance, update_address};
use lazy_static::lazy_static;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::{Arc, Mutex};
use rusqlite;
use tokio::runtime::Runtime;


// Set up the connection manager and pool
lazy_static! {
    static ref POOL: Arc<Mutex<r2d2::Pool<SqliteConnectionManager>>> = {
        let manager = SqliteConnectionManager::file("userstable.db");
        let pool = r2d2::Pool::builder()
            .build(manager)
            .unwrap_or_else(|_| panic!("Error creating pool"));
        Arc::new(Mutex::new(pool))
    };
}


// Database operations
fn tip_data(conn: &rusqlite::Connection, sender_id: u64, recipient_id: u64, amount: i32) -> Result<(), String> {
    // Ensure that the sender and recipient exist in the database
    let sender_exists = get_user(conn, sender_id);
    let recipient_exists = get_user(conn, recipient_id);
    let sender_balance = get_balance(conn, sender_id);

    // Update the balances of the sender and recipient in the database
    update_balance(conn, sender_id, -amount);
    update_balance(conn, recipient_id, amount);

    Ok(())
}

async fn update_data(conn: &rusqlite::Connection, user_id: u64, address: &str) -> Result<(), rusqlite::Error> {
    // Check if the user exists
    let user_exists = get_user(conn, user_id).await.is_ok();
    if !user_exists {
        // Insert a new row if the user doesn't exist
        insert_row(conn, "", &address, 0).await?;
    } else {
        // Update the row if the user exists
        update_address(conn, user_id, &address).await?;
    }
    Ok(())
}

pub async fn register_data(conn: &rusqlite::Connection, user_id: u64, address: &str) -> Result<(), rusqlite::Error> {
    // Check if the user exists
    let user_exists = get_user(conn, user_id).await.is_ok();
    if !user_exists {
        // Insert a new row if the user doesn't exist
        insert_row(conn, "", &address, 10);
    } else {
        // Update the row if the user exists
        update_address(conn, user_id, &address);
    }
    Ok(())
}

// bot commands
#[serenity::framework::standard::macros::group]
#[commands(tip, update, balance, register)]
pub struct General;

#[command]
pub async fn tip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Clone the connection pool and HTTP client
    let pool = POOL.clone();
    let http = ctx.http.clone();

    // Parse the user ID or username and amount from the command arguments
    let name = args.single::<String>()?;
    let amount = args.single::<i32>()?;

    // Look up the user ID of the recipient
    let recipient_id = match name.parse::<u64>() {
        Ok(id) => match http.get_user(id).await {
            Ok(user) => user.id,
            Err(_) => {
                msg.channel_id.say(&http, "User not found").await;
                return Err("User not found".into());
            }
        },
        Err(_) => {
            msg.channel_id.say(&http, "Invalid user ID or username").await;
            return Err("Invalid user ID or username".into());
        }
    };

    // Lock the connection pool and get a connection
    let conn = pool.lock().expect("Error acquiring mutex").get()?;

    // call tip_data function to handle the database operations
    tip_data(&conn, *msg.author.id.as_u64(), *recipient_id.as_u64(), amount);

    // Send a message to the channel indicating that the tip was successful
    let sender_name = msg.author.id.as_u64();
    let recipient_name = recipient_id.as_u64();
    msg.channel_id.say(&http, format!("{} tipped {} {}", sender_name, recipient_name, amount)).await;
    
    Ok(())
}

#[command]
pub async fn update(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Clone the connection pool and HTTP client
    let pool = POOL.clone();
    let http = ctx.http.clone();

    // Parse the user ID or address from the command arguments
    let address = args.single::<String>()?;

    // Lock the connection pool and get a connection
    let conn = pool.lock().expect("Error acquiring mutex").get()?;

    // call update_data function to handle the database operations
    update_data(&conn, *msg.author.id.as_u64(), &address);

    // Send a message to the channel indicating that the update was successful
    let user_name = msg.author.id.as_u64();
    msg.channel_id.say(&http, format!("{} address updated to {}", user_name, address)).await;
    
    Ok(())
}

#[command]
pub async fn balance(ctx: &Context, msg: &Message) -> CommandResult {
    // Clone the connection pool and HTTP client
    let pool = POOL.clone();
    let http = ctx.http.clone();

    // Lock the connection pool and get a connection
    let conn = pool.lock().expect("Error acquiring mutex").get()?;

    // Use the connection here
    let balance = {
        let result = get_balance(&conn, *msg.author.id.as_u64());

        let mut runtime = Runtime::new()?;
        runtime.block_on(result)?
    };

    msg.channel_id
        .say(&http, format!("Your balance is: {}", balance))
        .await?;

    Ok(())
}

#[command]
pub async fn register(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Clone the connection pool and HTTP client
    let pool = POOL.clone();
    let http = ctx.http.clone();

    // Get the address from the command arguments
    let address = args.single::<String>()?;

    // Lock the connection pool and get a connection
    let conn = pool.lock().expect("Error acquiring mutex").get()?;
    let mut runtime = Runtime::new()?;
    runtime.block_on(register_data(&conn, *msg.author.id.as_u64(), &address))?;

    // Send a message to the channel indicating that the registration was successful
    let user_name = msg.author.id.as_u64();
    msg.channel_id.say(&http, format!("{} registered with address: {}", user_name, address)).await?;

    Ok(())
}
