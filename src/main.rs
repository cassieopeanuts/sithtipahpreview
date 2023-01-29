
use serenity::async_trait;
mod modules;
use modules::commands::ALLCOMMS_GROUP;
use dotenv::dotenv;
use serenity::framework::StandardFramework;
use serenity::{
    client::Context,
    framework::standard::{
        Args
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!") {
            let mut args_vec: Vec<&str> = msg.content.split(" ").collect();
            let command = args_vec.remove(0);
            command.trim_start_matches("!");

            let args = Args::new(&args_vec.join(" "), &[",".into(), ";".into()]);

            match command {
                "tip" => {
                    tokio::spawn(async move {
                        match modules::commands::tip(&ctx, &msg, args).await {
                            Ok(_) => {}
                            Err(why) => println!("Error executing command: {:?}", why),
                        }
                    });
                },
                "register" => {
                    tokio::spawn(async move {
                        match modules::commands::register(&ctx, &msg, args).await {
                            Ok(_) => {}
                            Err(why) => println!("Error executing command: {:?}", why),
                        }
                    });
                },
                "update" => {
                    tokio::spawn(async move {
                        match modules::commands::update(&ctx, &msg, args).await {
                            Ok(_) => {}
                            Err(why) => println!("Error executing command: {:?}", why),
                        }
                    });
                },
                "balance" => {
                    tokio::spawn(async move {
                        match modules::commands::balance(&ctx, &msg, args).await {
                            Ok(_) => {}
                            Err(why) => println!("Error executing command: {:?}", why),
                        }
                    });
                },
                _ => println!("Command not found"),
            }
        }
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

    let mut client = Client::builder(token, GatewayIntents::all())
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

