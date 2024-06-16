mod config;
mod gpio_watcher;

use std::path::PathBuf;
use clap::Parser;

use config::Config;
use gpio_watcher::GpioWatcher;



#[derive(Parser, Debug)]
#[command(version, about = "Monitor GPIO for changes and update Home Assistant through MQTT", long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}


fn main() 
{
    let args = Args::parse();
    let config = load_config(&args);

    println!("{:?}", config);

    //mqtt::mqtt_start(config);
    let mut gpio_watcher = GpioWatcher::start(&config, pin_changed);

    // ...

    gpio_watcher.stop();
}


fn load_config(args: &Args) -> Config
{
    let config_file: PathBuf;

    if let Some(args_config_file) = &args.config
    {
        config_file = PathBuf::from(args_config_file);
    }
    else if let Ok(env_config_file) = std::env::var("HAGPIOMQTT_CONFIG")
    {
        config_file = PathBuf::from(env_config_file);
    }
    else
    {
        config_file = PathBuf::from("/etc/ha-gpio-mqtt.conf");
    }


    match Config::from_file(config_file.as_path())
    {
        Ok(config) =>
        {
            return config;
        },

        Err(e) =>
        {
            println!("Error while loading configuration from {}", config_file.display());
            println!("{}", e.message);
            std::process::exit(1);
        }
    }
}


fn pin_changed(pin: u8, level: gpio_watcher::GpioPinLevel)
{
    println!("Pin {} changed to {:?}", pin, level);
    // TODO send to MQTT module
}