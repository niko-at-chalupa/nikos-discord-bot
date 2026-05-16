use tokio::sync::RwLock;
use std::sync::Arc;

pub struct Data {
    pub rei_cache: RwLock<PostCache>,
    pub teto_cache: RwLock<PostCache>,
    pub spicyteto_cache: RwLock<PostCache>,
} // User data, which is stored and accessible in all command invocations

impl Data {
    pub fn new() -> Self {
        Self {
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
