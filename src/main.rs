use serenity::{
    model::{event::ResumedEvent, gateway::Ready, gateway::GatewayIntents},
    prelude::*,
};
use serenity::{
    client::Client,
};
use async_trait::*;
use tokio;
mod modules;
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
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create the Discord client
    let mut client = Client::builder(&token, GatewayIntents::GUILD_MEMBERS)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    
    //probably need a tip function call, but i hope not

    // Login to Discord and start the event loop
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}