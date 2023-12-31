use chrono::{Utc, DateTime, Duration};
use regex::Regex;
use reqwest;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_json::{Value};

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

mod chrono_duration {
    use super::*;

    pub fn serialize<S>(duration: &Duration, serializer: S) 
            -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_i64(duration.num_seconds())
    }

    pub fn deserialize<'de, D>(deserializer: D) 
            -> std::result::Result<Duration, D::Error> where D: Deserializer<'de> {
        let seconds = i64::deserialize(deserializer)?;
        Ok(Duration::seconds(seconds))
    }
}

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
    /// Time of last account data update
    #[serde(alias = "accountLastUpdate")]
    #[serde(with = "chrono::serde::ts_seconds")]
    account_last_update: DateTime<Utc>,

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

    /// Time account was created at
    #[serde(alias = "createdAt")]
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    created_at: DateTime<Utc>,

    /// Is character currently dead
    dead: bool,

    /// Total elites killed
    #[serde(alias = "elitesKilled")]
    elites_killed: u64,

    /// List of equipped items
    equipment: Vec<Item>,

    /// List of unexplored locations (?)
    fog_of_wars: Vec<String>,

    /// Total gold collected
    #[serde(alias = "goldCollected")]
    gold_collected: u64,

    /// Hardcore mode enabled
    hardcore: bool,

    /// Associated character ID
    id: String,

    /// Time of last login
    #[serde(alias = "lastLogin")]
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    last_login: DateTime<Utc>,

    /// Duplicate of `account_last_update`
    #[serde(alias = "lastUpdate")]
    last_update: u64,

    /// Level
    level: u64,

    /// Total monsters killed
    #[serde(alias = "monstersKilled")]
    monsters_killed: u64,

    /// Total players killed
    #[serde(alias = "playersKilled")]
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
    #[serde(alias = "secondsPlayed")]
    #[serde(with = "chrono_duration")]
    play_time: Duration,

    /// List of skill tree (?)
    #[serde(alias = "skillTree")]
    skill_tree: Vec<String>,

    /// List of enabled skills
    skills: Vec<Skill>,

    /// Associated twitch account
    twitch: Option<String>,

    /// List of waypoints (?)
    waypoints: Vec<String>,

    /// Current world tier (1-4)
    #[serde(alias = "worldTier")]
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
    #[serde(alias = "itemtype")]
    item_type: String,

    /// Name of the item
    name: String,

    /// Item parent ID (?)
    #[serde(alias = "parent")]
    parent_id: Option<u64>,

    /// Item power level
    power: u64,

    /// Quality level of the item (unique, legendary, etc)
    quality_level: String,
    quality_modifier: u64,

    /// Required level to equip the item
    required_level: u64,

    /// Item marked for junk (?)
    strikethrough_affix_ids: Vec<u64>,
    strikethrough_affixes: Vec<String>,

    /// Texture ID
    #[serde(alias = "tex")]
    texture_id: u64,
    
    /// Level of applied upgrades
    upgrades: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Skill {
    /// Description of the skill
    #[serde(alias = "desc")]
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
        let char_obj= character.as_object_mut().ok_or_else(|| {
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
            if !char_obj.contains_key(key) || &char_obj[key] != value {
                char_obj.insert(key.clone(), value.clone());
            }
        }

        // Clean HTML tags from all `base_affixes` and `added_affixes`
        if let Some(Value::Array(equipment)) = char_obj.get_mut("equipment") {
            for item in equipment.iter_mut() {
                Self::clean_affix_field(item, "base_affixes");
                Self::clean_affix_field(item, "added_affixes");
            }
        }
        
        Ok(())
    }

    /// Serialize the account data and save it to a file
    fn save_to_file(&self, account_id: u64) -> Result<()> {
        // Serialize the account to a prettified JSON string
        let serialized = serde_json::to_string_pretty(&self)?;

        // Assign the filename to be `account_{account_id}.json`
        let filename = format!("data/account_{}.json", account_id);

        // Write the serialized data to the file
        std::fs::write(&filename, serialized)?;

        println!("Account saved to file: {}", filename);
        Ok(())
    }

    /// Removes HTML tags from a given string
    fn remove_html_tags(text: &str) -> String {
        let re = Regex::new(r"</?[^>]+(>|$)").expect("Invalid regex pattern");
        re.replace_all(text, "").to_string()
    }

    /// Clean HTML tags from given affix field
    fn clean_affix_field(item: &mut Value, field: &str) {
        if let Some(Value::Array(affixes)) = item.get_mut(field) {
            for affix in affixes.iter_mut() {
                if let Some(text) = affix.as_str() {
                    *affix = Value::String(Self::remove_html_tags(text));
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let account = Account::parse(ACCOUNT_ID)?;
    println!("{:?}", account);

    // Save the account to a file
    account.save_to_file(ACCOUNT_ID)?;

    Ok(())
}
