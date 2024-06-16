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
    pub device_name: String,
    pub client_id: String,

    #[serde(default = "Config::default_discovery_prefix")]
    pub discovery_prefix: String,

    pub gpio: Vec<ConfigGpio>
}


#[derive(Deserialize, Debug)]
pub struct ConfigMqtt
{
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String
}


#[derive(Deserialize, Debug)]
pub struct ConfigGpio
{
    pub entity_id: String,
    pub name: String,
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
    fn default() -> Self { Self::None }
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


    pub fn default_discovery_prefix() -> String
    {
        String::from("homeassistant")
    }
}