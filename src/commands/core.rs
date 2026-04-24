use crate::types::Data;
use crate::types::Error;
use crate::types::Context;

pub async fn commands() -> Vec<poise::Command<Data, Error>> {
    println!("[core]");

    let commands = vec![
        ping()
    ];

    for command in &commands {
        println!("✓ {} - {}", command.name, command.description.as_deref().unwrap_or("[no description]"));
    }
    println!(); // for the whitespace between commands

    commands
}

/// Ping the bot, to check if it's running okay.
#[poise::command(slash_command)]
pub async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    ctx.send(poise::CreateReply::default()
        .ephemeral(true)
        .content("Pong!!!"),
    ).await?;
    Ok(())
}