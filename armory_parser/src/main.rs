use chrono::{DateTime, Utc};
use reqwest;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

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

    /// JSON is not a valid object
    JsonObject(String),

    /// IO error
    IOError(std::io::Error),
}

/// Implement the formatter for our custom error type
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::HttpRequest(e) => 
                write!(f, "HTTP request error: {}", e),
            Error::HttpResponseNonSuccess(e) =>
                write!(f, "HTTP response not successful: {}", e),
            Error::JsonParse(e) => 
                write!(f, "JSON parse error: {}", e),
            Error::JsonObject(e) =>
                write!(f, "JSON object error: {}", e),
            Error::IOError(e) =>
                write!(f, "IO error: {}", e),
        }
    }
}

/// Implement standard error trait and conversion from other error types
impl std::error::Error for Error {}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::HttpRequest(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonParse(err)
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

/// Custom Result type alias
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    /// Total bosses killed
    bosses_killed: u64,

    /// List of associated characters
    characters: Vec<Character>,

    /// Account clan ID
    clan_id: Option<String>,

    /// Account clan tag
    clan_tag: Option<String>,

    /// Total dungeons completed
    dungeons_completed: u64,

    /// Total players killed
    players_killed: u64,

    /// Linked twitch account
    twitch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Character {
    /// @TODO: Time of last account data update
    #[serde(rename = "accountLastUpdate")]
    account_last_update: u64,

    /// List of found Altars of Lilith
    altars: Vec<String>,

    /// Name
    name: String,

    /// Clan
    clan: Option<String>,

    /// Class
    class: String,

    /// List of completed quests
    completed_quests: Vec<String>,

    /// @TODO: Time account was created at
    #[serde(rename = "createdAt")]
    created_at: u64,

    /// @TODO: ??
    dead: bool,

    /// Total elites killed
    #[serde(rename = "elitesKilled")]
    elites_killed: u64,

    /// List of equipped items
    equipment: Vec<Item>,

    /// @TODO: ??
    fog_of_wars: Vec<String>,

    /// Total gold collected
    #[serde(rename = "goldCollected")]
    gold_collected: u64,

    /// Hardcore mode enabled
    hardcore: bool,

    /// Associated character ID
    id: String,

    /// @TODO: Time of last login
    #[serde(rename = "lastLogin")]
    last_login: u64,

    /// @TODO: Time of last character data update
    #[serde(rename = "lastUpdate")]
    last_update: u64,

    /// Level
    level: u64,

    /// Total monsters killed
    #[serde(rename = "monstersKilled")]
    monsters_killed: u64,

    /// Total players killed
    #[serde(rename = "playersKilled")]
    players_killed: u64,

    /// Average item power of equipment
    power: u64,

    /// Current position in queue (?)
    queue: u64,

    /// Associated Diablo season
    season: u64,

    /// Seasonal mode enabled
    seasonal: bool,

    /// Total time played
    #[serde(rename = "secondsPlayed")]
    play_time: u64,

    /// List of skill tree (?)
    #[serde(rename = "skillTree")]
    skill_tree: Vec<String>,

    /// List of enabled skills
    skills: Vec<Skill>,

    /// Associated twitch account
    twitch: Option<String>,

    /// List of waypoints (?)
    waypoints: Vec<String>,

    /// Current world tier (1-4)
    #[serde(rename = "worldTier")]
    world_tier: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    /// List of added affixes and their ids
    added_affix_ids: Vec<u64>,
    added_affixes: Vec<String>,

    /// List of base affixes and their ids
    base_affix_ids: Vec<u64>,
    base_affixes: Vec<String>,

    /// Item ID
    id: u64,

    /// Item type (helmet, chest, boots, etc)
    #[serde(rename = "itemtype")]
    item_type: String,

    /// Name of the item
    name: String,

    /// Parent of the item (?)
    parent: Option<u64>,

    /// Item power level
    power: u64,

    /// Quality level of the item (unique, legendary, etc)
    quality_level: String,
    quality_modifier: u64,

    /// Required level to equip the item
    required_level: u64,

    /// @TODO: ???
    strikethrough_affix_ids: Vec<u64>,
    strikethrough_affixes: Vec<String>,
    tex: u64,
    
    /// Level of applied upgrades
    upgrades: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Skill {
    /// Description of the skill
    #[serde(rename = "desc")]
    description: String,

    /// Name of the skill
    name: String,
}

impl Account {
    /// Parses account data from the D4Armory API for a given account ID
    fn parse(account_id: u64) -> Result<Self> {
        // Build the URL and fetch account data from the API as JSON
        let url = format!("{}/{}", BASE_URL, account_id);
        let mut account_data = Self::get_json(&url)?;

        // Process each character associated with the account
        if let Value::Array(characters) = &mut account_data["characters"] {
            for character in characters.iter_mut() {
                // Check if the character has an ID
                if let Value::String(character_id) = &character["id"] {
                    // Build the character detail URL
                    let url = format!("{}/{}", url, character_id);

                    // Fetch character data from the API
                    let mut character_data = Self::get_json(&url)?;

                    // Merge character details into the account's character
                    Self::merge_character(character, &mut character_data)?;
                }
            }
        }

        // Deserialize JSON data into Account struct
        Ok(serde_json::from_value(account_data)?)
    }
    
    /// Fetches JSON data from a given URL
    fn get_json(url: &str) -> Result<Value> {
        // Create an HTTP client and make a GET request to the URL
        let client = reqwest::blocking::Client::new();
        let response = client.get(url).send()?;

        // Process the HTTP response
        match response.status() {
            reqwest::StatusCode::OK => Ok(response.json()?),
            status => Err(Error::HttpResponseNonSuccess(status)),
        }
    }

    /// Merge character details into the account's character list
    fn merge_character(character: &mut Value, details: &mut Value) 
            -> Result<()> {
        // Ensure both inputs are objects before we merge them
        let character_obj = character.as_object_mut().ok_or_else(|| {
            Error::JsonObject(
                "Existing character entry is not a JSON object".to_string(),
            )
        })?;
        let details_obj = details.as_object().ok_or_else(|| {
            Error::JsonObject(
                "Character details entry is not a JSON object".to_string(),
            )
        })?;

        // Iterate over each field in the details entry
        for (key, value) in details_obj {
            // Update the field if it doesn't exist or has changed
            if !character_obj.contains_key(key) || 
                &character_obj[key] != value {
                character_obj.insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }

    /// Serialize the account data and save it to a file
    fn save_to_file(&self, account_id: u64) -> Result<()> {
        // Serialize the account to a prettified JSON string
        let serialized = serde_json::to_string_pretty(&self)?;

        // Assign the filename to be `account_{account_id}.json`
        let filename = format!("account_{}.json", account_id);

        // Write the serialized data to the file
        std::fs::write(&filename, serialized)?;

        println!("Account saved to file: {}", filename);
        Ok(())
    }
}

fn main() -> Result<()> {
    let account = Account::parse(ACCOUNT_ID)?;
    println!("{:?}", account);

    // Save the account to a file
    account.save_to_file(ACCOUNT_ID)?;

    Ok(())
}
