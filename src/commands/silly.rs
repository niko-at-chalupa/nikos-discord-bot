use poise::CreateReply;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateEmbedAuthor;
use crate::types::Data;
use crate::types::Error;
use crate::types::Context;
use pyo3::prelude::*;
use std::collections::HashSet;

pub async fn commands() -> Vec<poise::Command<Data, Error>> {
    println!("[silly]");

    let rule34_key = match std::env::var("RULE34_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("RULE34_API_KEY unset");
            String::new()
        },
    };
    let rule34_user_id = match std::env::var("RULE34_USER_ID") {
        Ok(key) => key,
        Err(_) => {
            println!("RULE34_USER_ID unset");
            String::new()
        },
    };

    let mut failed_commands = vec![];
    let mut commands = vec![
        teto()
    ];  
    if !rule34_key.is_empty() && !rule34_user_id.is_empty() {
        commands.push(spicyteto());
    } else {
        failed_commands.push(spicyteto());
    }

    for command in &commands {
        println!("{}✓ {} - {}", crate::ui::COLOR_SUCCESS, command.name, command.description.as_deref().unwrap_or("[no description]"));
    }
    for command in &failed_commands {
        println!("{}{} {} - {}", crate::ui::COLOR_ERROR, crate::ui::CROSS, command.name, command.description.as_deref().unwrap_or("[no description]"));
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

async fn get_post_from_rule34(query: &str) -> PyResult<PostData> {
    let query_str = query.to_string();
    let rule34_api_key = match std::env::var("RULE34_API_KEY") {
        Err(_) => panic!("Check for if the environment variable RULE34_API_KEY is present before using get_post_from_rule34"),
        Ok(key) => key,
    };
    let rule34_user_id = match std::env::var("RULE34_USER_ID") {
        Err(_) => panic!("Check for if the environment variable RULE34_USER_ID is present before using get_post_from_rule34"),
        Ok(key) => key,
    };

    tokio::task::spawn_blocking(move || {
        Python::attach(|py| {
            let rule34 = py.import("rule34")?;
            let client = rule34.getattr("client")?;

            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("limit", 1)?;
            let posts = client
                .getattr("Client")?
                .call1((rule34_api_key, rule34_user_id))?
                .call_method1("list_posts", (query_str,))?;

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

    let post = match get_post_from_safebooru("kasane_teto sort:random -ai* rating:general").await {
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

#[derive(Debug, poise::ChoiceParameter)]
enum Rating {
    #[name = "Questionable"]
    Questionable,
    #[name = "Explicit"]
    Explicit,
}

/// Get a spicy image of Kasane Teto from https://rule34.xxx
#[poise::command(slash_command, nsfw_only = true)]
pub async fn spicyteto(
    ctx: Context<'_>,
    #[description = "Rating"] rating: Rating,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let rating_tag = match rating {
        Rating::Explicit => "rating:explicit",
        Rating::Questionable => "rating:questionable",
    };

    let post = match get_post_from_rule34(format!("kasane_teto sort:random score:>=10 -ai* -scat -fart -video {rating_tag}").as_str()).await {
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