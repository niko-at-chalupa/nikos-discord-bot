pub mod types;
pub mod commands;
pub mod ui;

use poise::serenity_prelude as serenity;

use crate::types::Data;

#[tokio::main]
async fn main() {
    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let commands = crate::commands::all_commands().await;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logging in as {}", &ctx.cache.current_user().name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            }) // scope ends
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Shutting down...");
        shard_manager.shutdown_all().await;
    });

    client.start().await.unwrap();
}