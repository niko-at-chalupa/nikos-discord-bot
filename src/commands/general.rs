use poise::CreateReply;
use poise::serenity_prelude as serenity;
use crate::types::Data;
use crate::types::Error;
use crate::types::Context;

pub async fn commands() -> Vec<poise::Command<Data, Error>> {
    println!("[general]");

    let commands = vec![
        userid()
    ];

    for command in &commands {
        println!("{}✓ {} - {}", crate::ui::COLOR_SUCCESS, command.name, command.description.as_deref().unwrap_or("[no description]"));
    }
    println!("{}", crate::ui::COLOR_RESET); // for the whitespace + color reset between command groups

    commands
}

/// Get someone's (or your own) user ID
#[poise::command(slash_command)]
pub async fn userid(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let the_user = user.as_ref().unwrap_or(ctx.author());

    ctx.send(CreateReply::default()
        .ephemeral(true)
        .content(format!("{}", the_user.id.get())),
    ).await?;

    Ok(())
}