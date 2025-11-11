use std::collections::HashSet;

use sqlite::Connection;
use regex::Regex;

pub mod config;
use config::ConfigFile;

// https://stackoverflow.com/a/38461750
pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}


pub fn scan_for_triggers(configuration: &ConfigFile, title: &str, description: &str) -> Option<HashSet<String>> {
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


pub fn get_sql_match_count(url: &str, connection: &Connection) -> i64 {
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


pub fn mark_posted(url: &str, connection: &Connection) {
    let mut cursor = connection.prepare("INSERT INTO articles (url) VALUES(?)").unwrap().into_iter().bind((1, url)).unwrap();
    cursor.next();
}