use poise::CreateReply;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateEmbedAuthor;
use crate::types::Data;
use crate::types::Error;
use crate::types::Context;
use pyo3::prelude::*;
use pyo3::types::PyNone;
use std::collections::HashSet;
use std::str::FromStr;

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

#[derive(Debug)]
struct PostData {
    file_url: String,
    artists: Vec<String>,
}

async fn get_post_from_safebooru(query: &str) -> PyResult<PostData> {
    let query_str = query.to_string();

    tokio::task::spawn_blocking(move || {
        Python::attach(|py| {
            let safebooru = py.import("safebooru")?;
            let client = safebooru.getattr("client")?;

            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("limit", 1)?;
            let posts = client
                .getattr("list_posts")?
                .call((query_str,), Some(&kwargs))?;

            let post = posts.get_item(0)?;

            let file_url: String = post.getattr("file_url")?.extract()?;

            let artists: Vec<String> = {
                let tag_info = post.getattr("tag_info")?;
                if tag_info.is_none() {
                    vec![]
                } else {
                    tag_info
                        .getattr("artists")?
                        .extract::<HashSet<String>>()
                        .map(|s| s.into_iter().collect())?
                }
            };

            Ok(PostData { file_url, artists })
        })
    })
    .await
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
}

/// Get an image of Kasane Teto from https://safebooru.org
#[poise::command(slash_command)]
pub async fn teto(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let post = match get_post_from_safebooru("kasane_teto sort:random -ai*").await {
        Ok(post) => post,
        Err(e) => {
            ctx.send(CreateReply::default()
                .ephemeral(true)
                .content(format!("{}", crate::ui::ERROR))
            ).await?;
            println!("Error!! {}", e);
            return Ok(());
        }
    };

    ctx.send(CreateReply::default()
        .ephemeral(true)
        .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(post.artists.join(", ")))
            .image(post.file_url)
        )
    ).await?;

    Ok(())
}