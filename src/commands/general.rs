use crate::types::Data;
use crate::types::Error;
use crate::types::Context;

pub async fn commands() -> Vec<poise::Command<Data, Error>> {
    println!("Loading General commands");

    vec![
        ping()
    ]
}

#[poise::command(slash_command)]
pub async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .ephemeral(true)
        .content("Pong!!!"),
    ).await?;
    Ok(())
}