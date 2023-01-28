use serenity::{
    model::{event::ResumedEvent, gateway::Ready, gateway::GatewayIntents },
    prelude::*,
};

use serenity::client::{Client, Context};
use serenity::framework::standard::{
    StandardFramework,
};

use async_trait::*;
use tokio;
mod modules;
use std::*;
use dotenv::dotenv;

use crate::modules::ALLCOMMS_GROUP;

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
    dotenv().ok();

    let token = dotenv::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN environment variable was found");

    let application_id: u64 = dotenv::var("APPLICATION_ID")
        .expect("No APPLICATION_ID environment variable was found")
        .parse()
        .expect("APPLICATION_ID couldn't be parsed");
       
    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix("!")
            .allow_dm(false)
            .with_whitespace(true)
            .delimiters(vec![",", ";"])
            .case_insensitivity(true)
        )
        .group(&ALLCOMMS_GROUP);

    let mut client = Client::builder(token, GatewayIntents::empty())
        
        .framework(framework)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");


    // Login to Discord and start the event loop
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
