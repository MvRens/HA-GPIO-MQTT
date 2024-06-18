use std::{fs::File, path::Path};

use serde::Deserialize;


pub struct Error
{
    pub message: String
}


#[derive(Deserialize, Debug)]
pub struct Config
{
    pub mqtt: ConfigMqtt,
    pub homeassistant: ConfigHomeAssistant,
    pub gpio: Vec<ConfigGpio>
}


#[derive(Deserialize, Debug)]
pub struct ConfigMqtt
{
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub client_id: String
}


#[derive(Deserialize, Debug)]
pub struct ConfigHomeAssistant
{
    #[serde(default = "ConfigHomeAssistant::default_discovery_prefix")]
    pub discovery_prefix: String,

    #[serde(default = "ConfigHomeAssistant::default_birth_topic")]
    pub birth_topic: String,

    #[serde(default = "ConfigHomeAssistant::default_state_prefix")]
    pub state_prefix: String,

    pub device_name: String
}

#[derive(Deserialize, Debug)]
pub struct ConfigGpio
{
    pub entity_id: String,
    pub name: String,
    pub device_class: Option<String>,

    pub pin: u8,
    pub inverted: bool,

    #[serde(default = "ConfigGpioPull::default")]
    pub pull: ConfigGpioPull
}


#[derive(Deserialize, Debug)]
pub enum ConfigGpioPull
{
    None,
    Up,
    Down
}


impl ConfigGpioPull
{
    fn default() -> Self { Self::Up }
}


impl Config
{
    pub fn from_file(path: &Path) -> Result<Self, Error>
    {
        match File::open(path)
        {
            Ok(f) =>
            {
                match serde_yaml::from_reader(f)
                {
                    Ok(v) => Ok(v),
                    Err(e) => Err(Error { message: e.to_string() })
                }        
            },

            Err(e) => Err(Error { message: e.to_string() })
        }
    }
}


impl ConfigHomeAssistant
{
    pub fn default_discovery_prefix() -> String { String::from("homeassistant") }
    pub fn default_state_prefix() -> String { String::from("gpio") }
    pub fn default_birth_topic() -> String { String::from("homeassistant/status") }
}