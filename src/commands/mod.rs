use crate::types::Error;
use crate::types::Data;

pub mod general;
pub mod core;
pub mod silly;

pub async fn all_commands() -> Vec<poise::Command<Data, Error>> {
    println!("Registering commands...\n");

    let mut commands: Vec<poise::Command<Data, Error>> = vec![];

    commands.extend(general::commands().await);
    commands.extend(core::commands().await);
    commands.extend(silly::commands().await);

    commands
}