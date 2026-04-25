use poise::CreateReply;
use poise::serenity_prelude as serenity;
use crate::types::Data;
use crate::types::Error;
use crate::types::Context;
use pyo3::prelude::*;

pub async fn commands() -> Vec<poise::Command<Data, Error>> {
    println!("[silly]");

    let commands = vec![
        teto()
    ];

    for command in &commands {
        println!("{}✓ {} - {}", crate::ui::COLOR_SUCCESS, command.name, command.description.as_deref().unwrap_or("[no description]"));
    }
    println!("{}", crate::ui::COLOR_RESET); // for the whitespace + color reset between command groups

    commands
}

struct PostData {
    url: String,
    author: String,
    timestamp: u64,
}

fn get_post_from_safebooru(query: &Vec<&String>) -> PyResult<PostData> {
    unimplemented!()
}

/// Get an image of Kasane Teto from https://safebooru.org
#[poise::command(slash_command)]
pub async fn teto(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    ctx.send(CreateReply::default()
        .ephemeral(true)
        .content(format!("{}", crate::ui::ERROR)),
    ).await?;

    Ok(())
}