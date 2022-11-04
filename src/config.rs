use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct ConfigFile {
    pub mastodon: MastodonConfig,
    pub feeds: Vec<RSSFeedConfig>,
    pub content_warnings: Vec<ContentWarning>,
    pub filters: RSSFilters,
    pub persistence: Persistence,
}

#[derive(Deserialize)]
pub struct MastodonConfig {
    pub client_key: String,
    pub client_secret: String,
    pub client_token: String,
    pub account_id: String,
    pub api_url: String,
    pub base_url: String,
    pub redirect_url: String
}

#[derive(Deserialize)]
pub struct RSSFeedConfig {
    pub label: String,
    pub url: String
} 

#[derive(Deserialize)]
pub struct ContentWarning {
    pub label: String,
    pub phrases: Vec<String>
}


#[derive(Deserialize)]
pub struct RSSFilters {
    pub hashtags: Vec<String>
}

#[derive(Deserialize)]
pub struct Persistence {
    pub database_path: String
}