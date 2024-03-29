use clap::Parser;
use dissolve::strip_html_tags;
use elefren::Language;
use elefren::status_builder::Visibility;
use elefren::prelude::*;
use regex::Regex;
use rss::Channel;
use sqlite::Connection;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;

mod config;
use config::ConfigFile;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    #[arg(short, long, default_value = "false")]
    skip_mastodon: bool
}

const MAX_DESCRIPTION_LENGTH: usize = 300;

// https://stackoverflow.com/a/38461750
fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}


fn scan_for_triggers(configuration: &ConfigFile, title: &str, description: &str) -> Option<HashSet<String>> {
    let mut triggers = HashSet::new();

    for trigger_type in &configuration.content_warnings {
        if triggers.contains(&trigger_type.label) {
            continue
        }

        for phrase in &trigger_type.phrases {
            let re = Regex::new(&format!(r"(?i)\b{}s?\b", phrase)).unwrap();
            let phrase_found = re.is_match(title) || re.is_match(description);

            if phrase_found {
                triggers.insert(trigger_type.label.to_owned());
                break;
            } 
        }
    }

    if triggers.is_empty() {
        None
    } else {
        Some(triggers)
    }
}


fn get_sql_match_count(url: &str, connection: &Connection) -> i64 {
    let mut cursor = connection.prepare("SELECT COUNT(*) FROM articles WHERE url=?").unwrap().into_iter().bind((1, url)).unwrap();
    
    match cursor.next() {
        None => panic!("Count of matches returned none, rather than a number."),
        Some(r) => {
            match r {
                Ok(c) => c.read::<i64, _>(0),
                Err(e) => {
                    panic!("Database error when counting matches: {}", e);
                }
            }
        }
    }
}


fn mark_posted(url: &str, connection: &Connection) {
    let mut cursor = connection.prepare("INSERT INTO articles (url) VALUES(?)").unwrap().into_iter().bind((1, url)).unwrap();
    cursor.next();
}


fn main() {
    let args = Args::parse();

    let config_file = args.config;

    let configuration: ConfigFile = {
        let contents = match fs::read_to_string(config_file) {
            Ok(c) => c,
            Err(e) => {
                panic!("Unable to read configuration file: {}", e)
            }
        };

        match toml::from_str(&contents) {
            Ok(toml)  => toml,
            Err(e) => {
                panic!("Failed to parse config file: {}", e);
            }
        }
    };

    let post_visibility = match configuration.mastodon.visibility.to_uppercase().as_str() {
        "PUBLIC" => {
            Visibility::Public
        },
        "UNLISTED" => {
            Visibility::Unlisted
        },
        _ => {
            panic!("Invalid posting visibility specified in configuration, must be either public or unlisted.");
        }
    };

    let connection = sqlite::open(&configuration.persistence.database_path).unwrap();

    // does our persistence table exist?
    let mut cursor = connection.prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='articles'").unwrap().into_iter();
    
    let table_count = match cursor.next() {
        None => panic!("Count of tables returned none, rather than a number."),
        Some(r) => {
            match r {
                Ok(c) => c.read::<i64, _>(0),
                Err(e) => {
                    panic!("Database error when counting tables: {}", e);
                }
            }
        }
    };

    if table_count == 0 {
        connection.execute("CREATE TABLE articles (id INTEGER PRIMARY KEY, url TEXT)").unwrap();
        println!("Persistence table was not found, so it was created.");
    }

    let data: Data = Data{
        base: Cow::from(configuration.mastodon.base_url.to_owned()),
        client_id: Cow::from(configuration.mastodon.account_id.to_owned()),
        client_secret: Cow::from(configuration.mastodon.client_secret.to_owned()),
        redirect: Cow::from(configuration.mastodon.redirect_url.to_owned()),
        token: Cow::from(configuration.mastodon.client_token.to_owned()),
    };

    let mastodon = match args.skip_mastodon {
        true => {
            println!("Skipping Mastodon connection.");
            None
        },
        false => {
            let mastodon = Mastodon::from(data);
            if let Err(e) = mastodon.verify_credentials() {
                panic!("Unable to verify login credentials with Mastodon instance: {}", e);
            }
            println!("Mastodon credentials verified.");
            Some(mastodon)            
        }
    };

    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let user_agent_string = format!("{}/{}", NAME, VERSION);
    let client = reqwest::blocking::Client::builder().user_agent(user_agent_string).build().unwrap();

    for feed in &configuration.feeds {
        println!("Fetching {}", feed.label);
        let content = client.get(&feed.url).send().unwrap().bytes().unwrap();
        let channel = Channel::read_from(&content[..]).unwrap();
        
        for item in channel.items() {
            // have we already posted this?
            let this_url = match item.link() {
                None => continue,
                Some(url) => url
            };
            
            if get_sql_match_count(this_url, &connection) > 0 {
                println!("Skipping article we have already posted.");
                continue;
            }

            let mut this_title = match item.title() {
                None => continue,
                Some(t) => t.to_owned()
            };

            let upper_title = this_title.to_uppercase();

            let this_description = match item.description() {
                None => continue,
                Some(d) => d
            };

            let upper_description = this_description.to_uppercase();

            // check for content warnings
            let trigger_labels = scan_for_triggers(&configuration, &upper_title, &upper_description);
            
            let mut stripped_description = match item.description() {
                None => panic!("Description empty, impossible."),
                Some(d) => strip_html_tags(d).join::<&str>("\n")
            };

            // hashtag-ify
            for tag in &configuration.filters.hashtags {
                let re = Regex::new(&format!(r"(?i)\b(?P<text>{})\b", tag)).unwrap();
                let re_fmt = Regex::new(r"\W").unwrap();
                let formatted = format!("#{}", re_fmt.replace_all(tag, ""));

                stripped_description = re.replace_all(&stripped_description, &formatted).to_string();
                this_title = re.replace_all(&this_title, &formatted).to_string();
            }

            // ensure we stay under the max status length
            stripped_description = truncate(&stripped_description, MAX_DESCRIPTION_LENGTH).to_owned();

            // title, link, description, content warning
            let status = if let Some(tw) = trigger_labels {
                StatusBuilder::new()
                    .status(format!("Source: {}\n\n{}\n{}\n{}", feed.label, this_title, stripped_description, this_url))
                    .sensitive(false)
                    .spoiler_text(format!("CW: {}", tw.into_iter().collect::<Vec<String>>().join(",")))
                    .visibility(post_visibility)
                    .language(Language::Eng).build().unwrap()
            } else {
                StatusBuilder::new()
                    .status(format!("Source: {}\n\n{}\n{}\n{}", feed.label, this_title, stripped_description, this_url))
                    .sensitive(false)
                    .visibility(post_visibility)
                    .language(Language::Eng).build().unwrap()
            };
            
            if let Some(mastodon) = &mastodon {
                mastodon.new_status(status).unwrap();
                println!("Status posted.");
            } else {
                println!("Would have posted: {:?}", status);
            }
            
            mark_posted(this_url, &connection);
            println!("Marked as posted.");
        }
    }
}
