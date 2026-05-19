pub mod types;
pub mod commands;
pub mod ui;

use std::sync::Arc;
use std::fs;
use poise::serenity_prelude as serenity;

use crate::types::{Data, Config};
use crate::commands::silly::{get_posts_from_safebooru, get_posts_from_rule34};

#[tokio::main]
async fn main() {
    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let config_exists = fs::metadata("config.yaml").is_ok();
    let config_path = if config_exists {
        "config.yaml"
    } else {
        "example-config.yaml"
    };

    // Read and parse the YAML configuration file
    let config_str = fs::read_to_string(config_path)
        .expect("Failed to read configuration file");
    let config: Config = serde_yaml::from_str(&config_str)
        .expect("Failed to parse configuration file");

    let commands = crate::commands::all_commands().await;
    // Initialize shared state with the loaded configuration
    let data = Arc::new(Data::new(config));

    let data_for_framework = data.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands,
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logging in as {}", &ctx.cache.current_user().name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data_for_framework)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    // Background cache refresher
    let data_for_task = data.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(600));
        loop {
            interval.tick().await;
            println!("Refreshing caches...");

            // Rei Cache
            if let Ok(posts) = get_posts_from_safebooru("adachi_rei sort:random -ai* rating:general", 100).await {
                let mut cache = data_for_task.rei_cache.write().await;
                cache.posts = posts;
            }

            // Teto Cache
            if let Ok(posts) = get_posts_from_safebooru("kasane_teto sort:random -ai* rating:general", 100).await {
                let mut cache = data_for_task.teto_cache.write().await;
                cache.posts = posts;
            }

            // Spicy Teto Cache
            if let Ok(posts) = get_posts_from_rule34(format!("kasane_teto sort:random score:>=10 -ai* -scat -fart -video").as_str(), 100).await {
                let mut cache = data_for_task.spicyteto_cache.write().await;
                cache.posts = posts;
            }

            // Tetorei cache
            if let Ok(posts) = get_posts_from_safebooru(format!("kasane_teto adachi_rei 2girls sort:random").as_str(), 100).await {
                let mut cache = data_for_task.tetorei_cache.write().await;
                cache.posts = posts;
            }
        }
    });

    if config_exists {
        println!("Config loaded successfully!");
    } else {
        println!("Using example config!! Please make a config.yaml that fills everything that example-config does.");
    }


    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Shutting down...");
        shard_manager.shutdown_all().await;
    });

    client.start().await.unwrap();
}