use poise::CreateReply;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateEmbedAuthor;
use rand::seq::IndexedRandom;
use crate::types::{Data, Error, Context, TagInfo, PostData, PostProvider, PostCache};
use pyo3::prelude::*;
use std::collections::HashSet;
use poise::serenity_prelude as serenity;
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

async fn confirmation_check(ctx: Context<'_>, main_message: &str, _cancel_message: &str, yes_and_no_buttons_text: [&str; 2]) -> Result<bool, Error> {
    // 0. Check if the user is already confirmed for this session
    {
        let confirmed_users = ctx.data().confirmed_users.read().await;
        if confirmed_users.contains(&ctx.author().id) {
            return Ok(true);
        }
    }

    // 1. Send the ephemeral warning using Poise's CreateReply
    // This acknowledges the interaction if it hasn't been deferred yet.
    let reply = ctx.send(
        poise::CreateReply::default()
            .content(main_message)
            .ephemeral(true)
            .components(vec![
                serenity::CreateActionRow::Buttons(vec![
                    serenity::CreateButton::new("yes").label(yes_and_no_buttons_text[0]).style(serenity::ButtonStyle::Success),
                    serenity::CreateButton::new("no").label(yes_and_no_buttons_text[1]).style(serenity::ButtonStyle::Secondary),
                ])
            ])
    ).await?;

    // 2. Set up the message to wait for interactions on
    let message = reply.message().await?;

    // 3. Race them: button interaction OR text message OR timeout
    let confirmed = tokio::select! {
        // Wait for a button press on the ephemeral message (only from the author)
        Some(press) = message.await_component_interaction(ctx.serenity_context())
            .author_id(ctx.author().id)
            .next() => {
            let _ = press.defer(ctx.serenity_context()).await;
            press.data.custom_id == "yes"
        }
        // Wait for a "y"/"yes" or "n"/"no" message from the author in the same channel
        Some(msg) = serenity::collector::MessageCollector::new(ctx.serenity_context())
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .filter(|m| {
                let input = m.content.to_lowercase();
                input == "y" || input == "yes" || input == "n" || input == "no"
            })
            .next() => {
            let input = msg.content.to_lowercase();
            input == "y" || input == "yes"
        }
        // Fail-safe timeout after 15 seconds
        _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
            false
        }
    };

    // 4. Clean up the ephemeral message
    let _ = reply.delete(ctx).await;
    
    if confirmed {
        // 5. Save the preference for the rest of the session
        let mut confirmed_users = ctx.data().confirmed_users.write().await;
        confirmed_users.insert(ctx.author().id);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Helper to handle the bot channel confirmation and initial deferral
async fn get_privacy_and_defer(ctx: Context<'_>) -> Result<bool, Error> {
    let in_bot_channel = ctx.data().config.bot_channels.contains(&ctx.channel_id());
    
    if !in_bot_channel {
        ctx.defer_ephemeral().await?;
        return Ok(true);
    }

    let confirmed = {
        let confirmed_users = ctx.data().confirmed_users.read().await;
        confirmed_users.contains(&ctx.author().id)
    };

    if confirmed {
        ctx.defer().await?;
        return Ok(false);
    }

    let main_message = r#"You're in a bot channel, which means the output of this command will (by default) be shown to everybody in the channel!!
-# While this is okay for pretty much everybody, consent is somewhat dubious for how the message will be sent (you might think it's just you, but it's actually everybody).
```ansi
Use the [0;2m[1;2mbuttons below[0m to confirm[0m, or [1;2msay[0m [2;32my[0m[1;2m/[0m[1;2m[0m[2;31mn[0m [1;2min the chat[0m.[0;2m[0;2m[0m[0m
```
-# Your choice will be saved and recorded. You can't change this on your own."#;
    let cancel_message = "-# Timeout, but I'll send privately just in case...";
    let yes_and_no_buttons_text = ["Yes, I'm okay with my messages out there!", "No, keep it to myself"];
    
    if confirmation_check(ctx, main_message, cancel_message, yes_and_no_buttons_text).await? {
        Ok(false)
    } else {
        Ok(true)
    }
}

/// Get an image of Kasane Teto from https://safebooru.org
#[poise::command(slash_command)]
pub async fn teto(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let private = get_privacy_and_defer(ctx).await?;

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
        .ephemeral(private)
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
    let private = get_privacy_and_defer(ctx).await?;

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
        .ephemeral(private)
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
    let private = get_privacy_and_defer(ctx).await?;

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
        .ephemeral(private)
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
