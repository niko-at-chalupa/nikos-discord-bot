use crate::types::Error;
use crate::types::Data;

pub mod general;

pub async fn all_commands() -> Vec<poise::Command<Data, Error>> {
    println!("Loading commands...!!");

    let mut commands: Vec<poise::Command<Data, Error>> = vec![];

    commands.extend(general::commands().await);

    commands
}