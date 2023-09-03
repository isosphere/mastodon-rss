# mastodon-rss
An RSS feed reader for Mastodon that posts status to an account with content warnings and automatic hashtagging.

## Configuration
Create a `config.toml` file with the following structure, with your own values:

```toml
[mastodon]
client_key = ""
client_secret = ""
client_token = ""
account_id = ""
base_url = "https://oceanplayground.social"
redirect_url = "urn:ietf:wg:oauth:2.0:oob"
api_url = "https://oceanplayground.social/api/v1/"
visibility = "public"

[persistence]
database_path = "mastodon.db"

[filters]
hashtags = [
    "halifax", "moncton", "amherst", "cumberland", "irving",  "Nova Scotia Power",
    "cape breton", "new glasgow", "canada", "rcmp", "annapolis valley",
    "nova scotia", "prince edward island", "pei", "mental health", "provincial budget",
    "COVID-19", "province house", "housing", "moncton", "new brunswick",
    "NB Power", "first nation", "saint john", "prime minister", "Charlottetown",
    "st. john's", "labrador city", "sydney", "truro", "dartmouth", "peggys cove", "amherst",
    "fredericton", "summerside", "miramichi", "Newfoundland and Labrador", "Newfoundland",
    "Mi'kmaq"
]

[[content_warnings]]
label = "Crime"
phrases = [
    "victim", "carjack", "carjacking", "stolen", "break-in", "broken into", "assault", "looting"
]

[[content_warnings]]
label = "US pol"
phrases = ["trump", "obama", "U.S. election", "white house", "U.S. politics", "American politician"]

[[feeds]]
label = "A News Feed"
url = "https://news.feed/"
```

## Usage
The program supports a `--config` argument for overriding the default path to the configuration file, and a `--skip-mastodon` option that will skip connecting to your Mastodon server but still register news articles as posted in the database.
