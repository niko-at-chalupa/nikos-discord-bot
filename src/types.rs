pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

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
