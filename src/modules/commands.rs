use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::{command, group},
    },
    model::{channel::Message}
};

use serenity::prelude::Context;
use super::database::{insert_row, get_user, plus_balance, minus_balance, get_balance, update_address, create_db_conn, add_balance };
use regex::Regex;
use serenity::model::prelude::*;
// bot commands
#[group("allcomms")]
#[commands(tip, update, balance, register, giveme5, tiphelp)]
pub struct Allcomms;

#[command]
pub async fn tiphelp(ctx: &Context, msg: &Message) -> CommandResult {
    let mut help_message = "SeedTipah available commands: \n\n".to_owned();
    help_message += "!register - Register your address to be able to deposit or withdraw MoonSeeds\n";
    help_message += "!update - Update your address\n";
    help_message += "!balance - Check your current balance in MoonSeedling Bag\n";
    help_message += "!tip <@user> <amount> - Tip another user some MoonSeeds(use amount without decimals)\n";
    help_message += "!giveme5 - Give yourself 5 MoonSeeds if you dont have any\n";
    help_message += "!tiphelp - Show this help message\n";

    msg.channel_id.say(&ctx.http, help_message).await?;

    Ok(())
}

#[command]
pub async fn register(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let conn = create_db_conn();
    // Retrieve the address from the command arguments
    let address = match args.single::<String>() {
        Ok(address) => address,
        Err(_) => {
            let _ = msg.reply(&ctx, "Please provide a valid Ethereum address.").await?;
            return Ok(());
        }
    };
    
    let address_regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
    if !address_regex.is_match(&address) {
        println!("Address does not match regex: {}", address);
        let _ = msg.reply(&ctx, "Invalid Ethereum address, please provide a 42-character hexadecimal string starting with '0x'.")
            .await?;
        return Ok(());
    }

    // Retrieve the user ID from the message sender
    let user_id = msg.author.id.as_u64().to_string();

    // Insert the user ID and address into the database
    if let Err(e) = insert_row(conn.clone(), &user_id.to_string(), &address, 0).await {
        let _ = msg.reply(&ctx, "Failed to register address.").await?;
        println!("Error inserting row into database: {:?}", e);
        return Err("Failed to register address".into());
    } else {
        let _ = msg.reply(&ctx, "Address registered successfully.").await?;
    }

    Ok(())
}

#[command]
pub async fn balance(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let conn = create_db_conn();
    let user_id = msg.author.id.as_u64().to_string();

    // Retrieve the balance for the user from the database
    let balance = match get_balance(&conn, &user_id).await {
        Ok(balance) => balance,
        Err(e) => {
            let _ = msg.reply(&ctx, "Failed to retrieve balance.").await;
            println!("Error retrieving balance from database: {:?}", e);
            return Ok(());
        }
    };

    let _ = msg.reply(&ctx, format!("Your balance is {}.", balance)).await;
    Ok(())
}

#[command]
pub async fn update(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let conn = create_db_conn();

    // Check if the argument is a valid Ethereum address
    let address = match args.single::<String>() {
        Ok(address) => address,
        Err(_) => {
            let _ = msg.reply(&ctx, "Please provide a valid Ethereum address.").await?;
            return Ok(());
        }
    };
    
    let address_regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
    if !address_regex.is_match(&address) {
        let _ = msg.reply(&ctx, "Invalid Ethereum address, please provide a 42-character hexadecimal string starting with '0x'.")
        .await?;
        return Ok(());
    };
    //retrieve the UserId
    let user_id = msg.author.id.as_u64().to_string();
    
    //Update users address in the database
    if let Err(e) = update_address(&conn, &address, &user_id).await {
        let _ = msg.reply(&ctx, "Failed to update address.").await?;
        println!("Error updating row in database: {:?}", e);
    } else {
        let _ = msg.reply(&ctx, "Address updated successfully.").await?;
    }

    Ok(())
}

#[command]
pub async fn tip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
let conn = create_db_conn();

// Check if there are two arguments provided, the recipient's user ID and the amount to tip
if args.len() != 2 {
let _ = msg.reply(&ctx, "Incorrect number of arguments, please provide a user ID and an amount to tip.").await;
return Ok(());
};

let sender_id = msg.author.id.as_u64().to_string();

// Retrieve the recipient's user ID from the first argument
let recipient_id = match args.single::<UserId>() {
    Ok(user_id) => user_id,
    Err(_) => {
        let _ = msg.reply(&ctx, "Please provide a valid user ID to tip.").await;
        return Ok(());
    }
};

let recipient_id_str = recipient_id.to_string();


// Check if the sender and recipient aren't the same
if recipient_id_str == sender_id {
    let _ = msg.reply(&ctx, "You cannot tip yourself").await;
    return Ok(());
}


let amount = match args.single::<i32>() {
    Ok(amount) => amount,
    Err(_) => {
        let _ = msg.reply(&ctx, "Invalid tip amount, please provide a positive integer.").await;
        return Ok(());
    },
};

// Check if the message author has enough funds to tip
let sender_balance = match get_balance(&conn, &sender_id).await {
    Ok(balance) => balance,
    Err(e) => {
        let _ = msg.reply(&ctx, "Failed to retrieve sender's balance.").await;
        println!("Error retrieving sender's balance: {:?}", e);
        return Ok(());
    }
};
if sender_balance < amount {
    let _ = msg.reply(&ctx, "Insufficient balance to tip that amount.").await;
    return Ok(());
}

    // Check if the recipient's user ID exists in the database
    let recipient_exists = get_user(&conn, &recipient_id.to_string()).await.is_ok();
    if !recipient_exists {
        // Insert the recipient into the database with a None address
        let conn = create_db_conn();
        if let Err(e) = insert_row(conn.clone(), &recipient_id.to_string(), "adressnotprovided", 0).await {
            let _ = msg.reply(&ctx, "Failed to register recipient.").await;
            println!("Error inserting row into database: {:?}", e);
            return Ok(());
        }
    }

    // Subtract the tip amount from the sender's balance
    if let Err(e) = minus_balance(&conn, &sender_id.to_string(), amount).await {
        let _ = msg.reply(&ctx, "Failed to tip recipient.").await;
        println!("Error subtracting balance from sender: {:?}", e);
        return Ok(());
    }

    // Add the tip amount to the recipient's balance
    if let Err(e) = plus_balance(&conn, &recipient_id.to_string(), amount).await {
        let _ = msg.reply(&ctx, "Failed to tip recipient.").await;
        println!("Error adding balance to recipient: {:?}", e);
        // Roll back the sender's balance subtraction
        let _ = plus_balance(&conn, &sender_id.to_string(), amount).await;
        return Ok(());
    }

    let _ = msg.reply(&ctx, format!("Tipped {} to <@{}>.", amount, recipient_id)).await;

    println!("args: {:?}", args);
    println!("recipient_id: {:?}", recipient_id);
    println!("amount: {:?}", amount);
    
    Ok(())
}

#[command]
async fn giveme5(ctx: &Context, msg: &Message) -> CommandResult {
    let user_id = &msg.author.id.to_string();
    let conn = create_db_conn();
    let balance = get_balance(&conn, user_id).await?;

    if balance < 1 {
        add_balance(&conn, user_id).await?;
        let new_balance = get_balance(&conn, user_id).await?;
        let reply = format!("Here's your 5 MoonSeeds! Your balance is now {}", new_balance);
        msg.channel_id.say(&ctx.http, &reply).await?;
    } else {
        let reply = format!("Sorry, you already have enough MoonSeeds! Your balance is {}", balance);
        msg.channel_id.say(&ctx.http, &reply).await?;
    }

    Ok(())
}
