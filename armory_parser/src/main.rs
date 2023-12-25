use chrono::{DateTime, Utc};
use reqwest;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::{error, fmt, result};

/// Player's account ID
const ACCOUNT_ID: u64 = 370940626;

/// Base URL for D4Armory
const BASE_URL: &str = "https://d4armory.io/api/armory";

/// Events URL for D4Armory
const EVENTS_URL: &str = "https://d4armory.io/api/events/recent";

/// Custom error type
#[derive(Debug)]
enum Error {
    /// HTTP request error
    HttpRequest(reqwest::Error),

    /// HTTP response was not successful
    HttpResponseNonSuccess(reqwest::StatusCode),

    /// JSON parsing error
    JsonParse(serde_json::Error),

    /// JSON expected array
    JsonExpectedArray,
}

/// Implement the formatter for our custom error type
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HttpRequest(e) => 
                write!(f, "HTTP request error: {}", e),
            Error::HttpResponseNonSuccess(e) =>
                write!(f, "HTTP response not successful: {}", e),
            Error::JsonParse(e) => 
                write!(f, "JSON parse error: {}", e),
            Error::JsonExpectedArray =>
                write!(f, "JSON expected array"),
        }
    }
}

/// Implement standard error trait and conversion from other error types
impl error::Error for Error {}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::HttpRequest(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonParse(err)
    }
}

/// Custom Result type alias
type Result<T> = result::Result<T, Error>;

/// Character world tier (1-4)
#[derive(Debug, Serialize, Deserialize)]
struct WorldTier(u32);

#[derive(Debug, Serialize, Deserialize)]
struct Hero {
    /// Total play time
    play_time: chrono::Duration,

    /// Last time played
    last_time_played: DateTime<Utc>,

    /// Monsters killed
    monsters_killed: usize,

    /// Elites killed
    elites_killed: usize,

    /// Total gold collected
    gold_collected: usize,

    /// Power
    power: usize,

    /// Current world tier
    world_tier: WorldTier,
}

#[derive(Debug)]
struct Character {
    /// Name
    name: String,

    /// ID
    id: String,

    /// Class
    class: String,

    /// Level
    level: u64,

    /// Time of last data update
    last_update: DateTime<Utc>,

    /// Hardcore mode enabled
    hardcore: bool,

    /// Seasonal mode enabled
    seasonal: bool,
}

#[derive(Debug)]
struct Account {
    /// Account ID
    account_id: u64,

    /// Dungeons completed
    dungeons_completed: u64,

    /// Players killed
    players_killed: u64,

    /// Clan ID
    clan_id: String,

    /// Clan tag
    clan_tag: String,

    /// Twitch
    twitch: String,

    /// List of characters
    characters: Vec<Character>,
}

impl Account {
    fn parse(account_id: u64) -> Result<Self> {
        let url = format!("{}/{}", BASE_URL, account_id);
        let data = Self::get_json(&url)?;

        let dungeons_completed = data["dungeons_completed"].as_u64().unwrap();
        let players_killed = data["players_killed"].as_u64().unwrap();
        let clan_id = data["clan_id"].to_string();
        let clan_tag = data["clan_tag"].to_string();
        let twitch = data["twitch"].to_string();
        let characters = data["characters"].as_array()
            .ok_or(Error::JsonExpectedArray)?;

        let characters: Vec<Character> = characters.iter().map(|character| {
            let character_id = character["id"].as_str()
                .unwrap_or_default().to_string();

            let url = format!("{}/{}/{}", url, account_id, character_id);
            println!("url: {}", url);

            Character {
                name: character["name"].to_string(),
                id: character_id,
                class: character["class"].to_string(),
                level: character["level"].as_u64().unwrap(),
                last_update: DateTime::parse_from_rfc3339(
                    character["lastUpdate"].as_str().unwrap()).unwrap()
                    .with_timezone(&Utc),
                hardcore: character["hardcore"].as_bool().unwrap(),
                seasonal: character["seasonal"].as_bool().unwrap(),
            }
        }).collect();

        Ok(Account { 
            account_id, dungeons_completed, players_killed, clan_id, 
            clan_tag, twitch, characters 
        })
    }
    
    fn get_json(url: &str) -> Result<serde_json::Value> {
        let client = reqwest::blocking::Client::new();
        let response = client.get(url).send()?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(response.json()?),
            status => Err(Error::HttpResponseNonSuccess(status)),
        }
    }
}

fn main() -> Result<()> {
    let account = Account::parse(ACCOUNT_ID)?;
    println!("{:?}", account);
    Ok(())
}
