use tokio::sync::RwLock;
use std::sync::Arc;
use serde::Deserialize;
use poise::serenity_prelude as serenity;

/// Configuration structure representing the YAML config file
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// List of user IDs authorized to use sensitive commands
    pub authorized_users: Vec<serenity::UserId>,
    /// List of channel IDs where the bot is allowed to operate
    pub bot_channels: Vec<serenity::ChannelId>,
}

/// User data shared across all command invocations
pub struct Data {
    /// Global bot configuration
    pub config: Config,
    /// Cache for Rei Ayanami posts
    pub rei_cache: RwLock<PostCache>,
    /// Cache for Kasane Teto posts
    pub teto_cache: RwLock<PostCache>,
    /// Cache for explicit Kasane Teto posts
    pub spicyteto_cache: RwLock<PostCache>,
}

impl Data {
    /// Creates a new instance of Data with the provided configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            rei_cache: RwLock::new(PostCache::empty("rei".to_string(), 10, PostProvider::Safebooru)),
            teto_cache: RwLock::new(PostCache::empty("teto".to_string(), 10, PostProvider::Safebooru)),
            spicyteto_cache: RwLock::new(PostCache::empty("teto rating:explicit".to_string(), 10, PostProvider::Rule34)),
        }
    }
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

#[derive(Debug, Clone)]
pub struct TagInfo {
    pub general: Vec<String>,
    pub meta: Vec<String>,
    pub artists: Vec<String>,
    pub characters: Vec<String>,
    pub copyrights: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PostData {
    pub file_url: String,
    pub artists: Vec<String>,
    pub post_id: u64,
    pub tag_info: TagInfo,
}

#[derive(Debug, Clone)]
pub enum PostProvider {
    Safebooru,
    Rule34,
}

#[derive(Debug, Clone)]
pub struct PostCache {
    pub posts: Vec<PostData>,
    pub query: String,
    pub limit: usize,
    pub provider: PostProvider,
}

impl PostCache {
    pub fn empty(query: String, limit: usize, provider: PostProvider) -> Self {
        Self {
            posts: Vec::new(),
            query,
            limit,
            provider,
        }
    }
}
