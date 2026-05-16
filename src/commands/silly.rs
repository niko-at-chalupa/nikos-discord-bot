use poise::CreateReply;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateEmbedAuthor;
use rand::seq::IndexedRandom;
use crate::types::{Data, Error, Context, TagInfo, PostData, PostProvider, PostCache};
use pyo3::prelude::*;
use std::collections::HashSet;
use std::process::Output;
use std::sync::Arc;

pub async fn commands() -> Vec<poise::Command<Arc<Data>, Error>> {
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

    let which_fastfetch = tokio::task::spawn_blocking(|| {
        which::which("fastfetch")
    }).await;
    match which_fastfetch {
        Ok(_) => commands.push(fastfetch()),
        Err(e) => {
            failed_commands.push(fastfetch());
            println!("{}", e);
        }
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


impl PostCache {
    pub async fn new(query: String, limit: usize, provider: PostProvider) -> Result<PostCache, Error> {
        let mut new = PostCache { 
            posts: vec![],
            query: query,
            limit: limit,
            provider: provider,
        };
        let result = new.fill().await;
        match result {
            Ok(_) => Ok(new),
            Err(e) => Err(e)
        }
    }

    pub async fn fill(&mut self) -> Result<(), Error> {
        let result = self.get_posts();
        match result.await {
            Ok(posts) => {
                self.posts = posts;
                Ok(())
            },
            Err(e) => Err(e.into())
        }
    }

    pub async fn posts(&self) -> &Vec<PostData> {
        &self.posts
    }

    pub async fn pull_random(&self) -> Option<&PostData> {
        if self.posts.is_empty() {
            return None
        }
        let mut rng = rand::rng();
        self.posts.choose(&mut rng)
    }

    async fn get_posts(&self) -> Result<Vec<PostData>, Error> {
        let provider = &self.provider;
        let query = &self.query;
        let limit = self.limit;
        let result = match provider {
            PostProvider::Safebooru => get_posts_from_safebooru(&query.to_string(), limit).await,
            PostProvider::Rule34 => get_posts_from_rule34(&query.to_string(), limit).await,
        };
        match result {
            Ok(posts) => Ok(posts),
            Err(e) => Err(e.into()),
        }
    }
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

pub async fn get_posts_from_safebooru(query: &str, limit: usize) -> PyResult<Vec<PostData>> {
    let query_str = query.to_string();

    tokio::task::spawn_blocking(move || {
        Python::attach(|py| {
            let safebooru = py.import("safebooru")?;
            let client = safebooru.getattr("client")?;

            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("limit", limit)?;
            let posts = client
                .getattr("list_posts")?
                .call((query_str,), Some(&kwargs))?;

            let len = posts.len()?;
            let mut result = Vec::with_capacity(len);

            for i in 0..len {
                let post = posts.get_item(i)?;

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

                result.push(PostData { file_url, artists, post_id, tag_info });
            }

            Ok(result)
        })
    })
    .await
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
}

pub async fn get_posts_from_rule34(query: &str, limit: usize) -> PyResult<Vec<PostData>> {
    let query_str = query.to_string();
    let rule34_api_key = match std::env::var("RULE34_API_KEY") {
        Err(_) => panic!("Check for if the environment variable RULE34_API_KEY is present before using get_posts_from_rule34"),
        Ok(key) => key,
    };
    let rule34_user_id = match std::env::var("RULE34_USER_ID") {
        Err(_) => panic!("Check for if the environment variable RULE34_USER_ID is present before using get_posts_from_rule34"),
        Ok(key) => key,
    };

    tokio::task::spawn_blocking(move || {
        Python::attach(|py| {
            let rule34 = py.import("rule34")?;
            let client = rule34.getattr("client")?;

            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("limit", limit)?;
            let posts = client
                .getattr("Client")?
                .call1((rule34_api_key, rule34_user_id))?
                .call_method1("list_posts", (query_str,))?;

            let len = posts.len()?;
            let mut result = Vec::with_capacity(len);

            for i in 0..len {
                let post = posts.get_item(i)?;

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

                result.push(PostData { file_url, artists, post_id, tag_info });
            }

            Ok(result)
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

    let post = {
        let cache = ctx.data().teto_cache.read().await;
        cache.pull_random().await.cloned()
    };

    let post = match post {
        Some(post) => post,
        None => {
            ctx.send(CreateReply::default()
                .ephemeral(true)
                .content("Cache empty, please wait a moment!")
            ).await?;
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

    let post = {
        let cache = ctx.data().rei_cache.read().await;
        cache.pull_random().await.cloned()
    };

    let post = match post {
        Some(post) => post,
        None => {
            ctx.send(CreateReply::default()
                .ephemeral(true)
                .content("Cache empty, please wait a moment!")
            ).await?;
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

/// Get a spicy image of Kasane Teto from https://rule34.xxx
#[poise::command(slash_command, nsfw_only = true)]
pub async fn spicyteto(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let post = {
        let cache = ctx.data().spicyteto_cache.read().await;
        cache.pull_random().await.cloned()
    };

    let post = match post {
        Some(post) => post,
        None => {
            ctx.send(CreateReply::default()
                .ephemeral(true)
                .content("Cache empty, please wait a moment!")
            ).await?;
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

/// Run fastfetch on the server
#[poise::command(slash_command)]
pub async fn fastfetch(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let result = tokio::process::Command::new("fastfetch")
        .arg("--logo-type")
        .arg("none")
        .output() 
        .await;

    let output = match result {
        Ok(output) => output,
        Err(e) => {
            ctx.send(CreateReply::default()
                .ephemeral(true)
                .content(format!("{}", crate::ui::ERROR))
            ).await?;
            println!("Error!! {}", e);
            return Ok(());
        }
    };

    let stdout_content = String::from_utf8_lossy(&output.stdout).replace("`", "\\`");
    ctx.send(CreateReply::default()
        .ephemeral(true)
        .content(format!("```ansi\n{}\n```", stdout_content))
    ).await?;

    Ok(())
}