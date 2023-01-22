use modules::commands;
use serenity::{
    model::{event::ResumedEvent, gateway::Ready, gateway::GatewayIntents},
    prelude::*, framework::standard::Command,
};
use serenity::{
    client::Client,
    framework::standard::StandardFramework,
};
use async_trait::*;
use tokio;
mod modules;
use std::*;
use dotenv;


struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn resume(&self, _ctx: Context, _: ResumedEvent) {
        println!("Resumed");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the Discord token from the environment
    let token = dotenv::var("DISCORD_TOKEN").expect("Expected a token in the environment");

        // Initialize the command framework
        let mut framework = StandardFramework::new()
        .configure(|c| c
            .prefix("!")
            .allow_dm(false)
            .with_whitespace(false)
            .delimiters(vec![",", ";"])
        );
    // Create the Discord client
    let mut client = Client::builder(&token, GatewayIntents::GUILD_MEMBERS)
    .event_handler(Handler)
    .framework(framework)
    .await
    .expect("Error creating client");

    // Login to Discord and start the event loop
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}