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
    pub gpio: Vec<ConfigGpio>
}


#[derive(Deserialize, Debug)]
pub struct ConfigMqtt
{
    pub hostname: String,
    pub port: u16
}


#[derive(Deserialize, Debug)]
pub struct ConfigGpio
{
    pub entity_id: String,
    pub name: String,
    pub pin: u8,
    pub inverted: bool
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