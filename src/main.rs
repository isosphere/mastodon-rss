use serde_derive::Deserialize;
use std::fs;
use toml;

#[derive(Deserialize)]
struct ConfigFile {
    mastodon: MastodonConfig,
    feeds: Vec<RSSFeedConfig>
}

#[derive(Deserialize)]
struct MastodonConfig {
    access_token: String,
    account_id: String,
    api_url: String
}

#[derive(Deserialize)]
struct RSSFeedConfig {
    label: String,
    url: String
}


fn main() {
    let config_file = "config.toml";

    let contents = match fs::read_to_string(config_file) {
        Ok(c) => c,
        Err(e) => {
            panic!("Unable to read configuration file: {}", e)
        }
    };

    let configuration: ConfigFile = match toml::from_str(&contents) {
        Ok(toml)  => toml,
        Err(e) => {
            panic!("Failed to parse config file: {}", e);
        }
    };
}
