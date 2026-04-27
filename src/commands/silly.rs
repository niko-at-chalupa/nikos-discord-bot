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
        teto(),
        rei(),
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
struct TagInfo {
    general: Vec<String>,
    meta: Vec<String>,
    artists: Vec<String>,
    characters: Vec<String>,
    copyrights: Vec<String>,
}

#[derive(Debug)]
struct PostData {
    file_url: String,
    artists: Vec<String>,
    post_id: u64,
    tag_info: TagInfo,
}

fn format_tags_with_ansi(tag_info: &TagInfo) -> String {
    const GENERAL: &str = "\x1b[34m";
    const META: &str = "\x1b[33m";
    const ARTISTS: &str = "\x1b[31m";
    const CHARACTERS: &str = "\x1b[32m";
    const COPYRIGHTS: &str = "\x1b[35m";
    const RESET: &str = "\x1b[0m";

    let mut output = String::from("```ansi\n");
    
    if !tag_info.general.is_empty() {
        output.push_str(&format!("{}{}{} ", GENERAL, tag_info.general.join(" "), RESET));
    }
    if !tag_info.meta.is_empty() {
        output.push_str(&format!("{}{}{} ", META, tag_info.meta.join(" "), RESET));
    }
    if !tag_info.artists.is_empty() {
        output.push_str(&format!("{}{}{} ", ARTISTS, tag_info.artists.join(" "), RESET));
    }
    if !tag_info.characters.is_empty() {
        output.push_str(&format!("{}{}{} ", CHARACTERS, tag_info.characters.join(" "), RESET));
    }
    if !tag_info.copyrights.is_empty() {
        output.push_str(&format!("{}{}{} ", COPYRIGHTS, tag_info.copyrights.join(" "), RESET));
    }
    
    output.push_str("```");
    output
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

            let post_id: u64 = post.getattr("post_id")?.extract()?;

            let tag_info_obj = post.getattr("tag_info")?;
            let (artists, tag_info) = if tag_info_obj.is_none() {
                (vec![], TagInfo {
                    general: vec![],
                    meta: vec![],
                    artists: vec![],
                    characters: vec![],
                    copyrights: vec![],
                })
            } else {
                let general: Vec<String> = tag_info_obj
                    .getattr("general")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let meta: Vec<String> = tag_info_obj
                    .getattr("meta")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let artists_set: Vec<String> = tag_info_obj
                    .getattr("artists")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let characters: Vec<String> = tag_info_obj
                    .getattr("characters")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let copyrights: Vec<String> = tag_info_obj
                    .getattr("copyrights")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                (artists_set.clone(), TagInfo {
                    general,
                    meta,
                    artists: artists_set,
                    characters,
                    copyrights,
                })
            };

            Ok(PostData { file_url, artists, post_id, tag_info })
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

            let post_id = post.getattr("post_id")?.extract()?;

            let file_url: String = post.getattr("file_url")?.extract()?;

            let tag_info_obj = post.getattr("tag_info")?;
            let (artists, tag_info) = if tag_info_obj.is_none() {
                (vec![], TagInfo {
                    general: vec![],
                    meta: vec![],
                    artists: vec![],
                    characters: vec![],
                    copyrights: vec![],
                })
            } else {
                let general: Vec<String> = tag_info_obj
                    .getattr("general")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let meta: Vec<String> = tag_info_obj
                    .getattr("meta")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let artists_set: Vec<String> = tag_info_obj
                    .getattr("artists")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let characters: Vec<String> = tag_info_obj
                    .getattr("characters")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                let copyrights: Vec<String> = tag_info_obj
                    .getattr("copyrights")?
                    .extract::<HashSet<String>>()
                    .map(|s| s.into_iter().collect())?;
                (artists_set.clone(), TagInfo {
                    general,
                    meta,
                    artists: artists_set,
                    characters,
                    copyrights,
                })
            };

            Ok(PostData { file_url, artists, post_id, tag_info })
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

    let artists = if !post.artists.is_empty() {
        post.artists.join(", ")
    } else {
        "[no artist tags]".to_string()
    };

    let tags_display = format_tags_with_ansi(&post.tag_info);

    ctx.send(CreateReply::default()
        .ephemeral(true)
        .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(artists))
            .description(tags_display)
            .image(post.file_url)
        )
    ).await?;

    Ok(())
}

/// Get an image of Adachi Rei from https://safebooru.org
#[poise::command(slash_command)]
pub async fn rei(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let post = match get_post_from_safebooru("adachi_rei sort:random -ai* rating:general").await {
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

    let artists = if !post.artists.is_empty() {
        post.artists.join(", ")
    } else {
        "[no artist tags]".to_string()
    };

    let tags_display = format_tags_with_ansi(&post.tag_info);

    ctx.send(CreateReply::default()
        .ephemeral(true)
        .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(artists))
            .description(tags_display)
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

    let artists = if !post.artists.is_empty() {
        post.artists.join(", ")
    } else {
        "[no artist tags]".to_string()
    };

    let tags_display = format_tags_with_ansi(&post.tag_info);

    ctx.send(CreateReply::default()
        .ephemeral(true)
        .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(artists))
            .description(tags_display)
            .image(post.file_url)
        )
    ).await?;

    Ok(())
}