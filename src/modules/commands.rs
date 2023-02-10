use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::{command, group},
    },
    model::{channel::Message, prelude::UserId,}
};
use serenity::prelude::Context;
use super::database::{insert_row, get_user, plus_balance, minus_balance, get_balance, update_address, create_db_conn };
use lazy_static::lazy_static;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::{Arc, Mutex};
use rusqlite;
use regex::Regex;

// bot commands
#[group("allcomms")]
#[commands(tip, update, balance, register)]
pub struct Allcomms;

#[command]
pub async fn register(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
   
    let conn = create_db_conn();
    // Check if there's a single argument provided, the Ethereum address
    if args.len() != 1 {
        let _ = msg.reply(&ctx, "Incorrect number of arguments, please provide a single Ethereum address.");
        return Ok(());
    }

    // Check if the argument is a valid Ethereum address
    let address = match args.single::<String>() {
        Ok(address) => address,
        Err(_) => {
            // send an error message indicating that the user must provide a valid address
            return Ok(());
        },
    };
    
    let address_regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
    if !address_regex.is_match(&address) {
        let _ = msg.reply(&ctx, "Invalid Ethereum address, please provide a 42-character hexadecimal string starting with '0x'.");
        return Ok(());
    }

    // If the argument is a valid Ethereum address, proceed to insert it into the database
    let user_id = msg.author.id.as_u64().to_string();
    match insert_row(conn, &user_id, &address, 10).await {
        Ok(_) => {
            let _ = msg.reply(&ctx, "Successfully registered Ethereum address.");
        }
        Err(e) => {
            let _ = msg.reply(&ctx, &format!("Failed to register Ethereum address: {}", e));
        }
    }

    Ok(())
}

#[command]
pub async fn balance(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let conn = create_db_conn();
    let user_id = msg.author.id.as_u64().to_string();

    match get_balance(&conn, &user_id).await {
        Ok(balance) => {
            let _ = msg.reply(&ctx, &format!("Your balance is {}", balance)).await;
        }
        Err(e) => {
            let _ = msg.reply(&ctx, &format!("Failed to retrieve your balance: {}", e)).await;
        }   
    }
    Ok(())
}

#[command]
pub async fn update(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let conn = create_db_conn();
    // Check if there's a single argument provided, the Ethereum address
    if args.len() != 1 {
        let _ = msg.reply(&ctx, "Incorrect number of arguments, please provide a single Ethereum address.");
        return Ok(());
    }

    // Check if the argument is a valid Ethereum address
    let address = match args.single::<String>() {
        Ok(address) => address,
        Err(_) => {
            // send an error message indicating that the user must provide a valid address
            return Ok(());
        },
    };
    
    let address_regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
    if !address_regex.is_match(&address) {
        let _ = msg.reply(&ctx, "Invalid Ethereum address, please provide a 42-character hexadecimal string starting with '0x'.");
        return Ok(());
    };

    let user_id = msg.author.id.as_u64().to_string();
    match update_address(&conn, &user_id, &address).await {
        Ok(_) => {
            let _ = msg.reply(&ctx, "Address updated successfully.").await;
        }
        Err(e) => {
            let _ = msg.reply(&ctx, &format!("Failed to update address: {}", e)).await;
        }
    }
    Ok(())
}

#[command]
pub async fn tip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
let mut conn = create_db_conn();
// Check if there are two arguments provided, the recipient's user ID and the amount to tip
if args.len() != 2 {
let _ = msg.reply(&ctx, "Incorrect number of arguments, please provide a user ID and an amount to tip.").await;
return Ok(());
};

let recipient_id = match args.single::<String>() {
    Ok(recipient_id) => recipient_id,
    Err(_) => {
        let _ = msg.reply(&ctx, "Invalid user ID, please mention a valid user in the form of @user.").await;
        return Ok(());
    },
};

let amount = match args.single::<i32>() {
    Ok(amount) => amount,
    Err(_) => {
        let _ = msg.reply(&ctx, "Invalid tip amount, please provide a positive integer.").await;
        return Ok(());
    },
};

let sender_id = msg.author.id.as_u64().to_string();

// Check if the sender has enough balance
let sender_balance = match get_balance(&conn, &sender_id).await {
    Ok(balance) => balance,
    Err(e) => {
        let _ = msg.reply(&ctx, &format!("Failed to retrieve your balance: {}", e)).await;
        return Ok(());
    }
};

if sender_balance < amount {
    let _ = msg.reply(&ctx, "You don't have enough balance to make this tip.").await;
    return Ok(());
}

// Check if the recipient exists in the database
let recipient_exists = match get_user(&conn, &recipient_id.as_str()).await {
    Ok(exists) => exists,
    Err(e) => {
        let _ = msg.reply(&ctx, &format!("Failed to check if the recipient exists: {}", e)).await;
        return Ok(());
    }
};

// If the recipient doesn't exist, insert them into the database
if !recipient_exists {
    match insert_row(conn, &recipient_id.as_str(), "", 10).await {
        Ok(_) => (),
        Err(e) => {
            let _ = msg.reply(&ctx, &format!("Failed to insert the recipient into the database: {}", e)).await;
            return Ok(());
        }
    }
}

    // Clone the connection after it has been moved
    let conn = create_db_conn();
// Perform the tip
match minus_balance(&conn, &sender_id, amount).await {
    Ok(_) => (),
    Err(e) => {
        let _ = msg.reply(&ctx, &format!("Failed to subtract balance from the sender: {}", e)).await;
        return Ok(());
    }
};

// Add the tip to the recipient's balance
match plus_balance(&conn, &recipient_id, amount).await {
    Ok(_) => {
    let _ = msg.reply(&ctx, &format!("{} tokens tipped successfully to {}", amount, recipient_id)).await;
    }
    Err(e) => {
    let _ = msg.reply(&ctx, &format!("Failed to add balance to the recipient: {}", e)).await;
    return Ok(());
    }
    };
    
    Ok(())
}