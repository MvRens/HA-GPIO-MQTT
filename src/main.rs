mod config;
mod gpio_watcher;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use clap::Parser;
use env_logger::Env;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
            .format_timestamp(None)
            .init();
    
    let args = Args::parse();
    let config = load_config(&args);

    println!("{:?}", config);

    //mqtt::mqtt_start(config);
    let mut gpio_watcher = GpioWatcher::start(&config);

    // TODO monitor for signals to stop service

    while match gpio_watcher.poll()
    {
        gpio_watcher::GpioPollResult::None =>
        {
            thread::sleep(Duration::from_millis(10));
            true
        },

        gpio_watcher::GpioPollResult::PinChanged(pin, level) =>
        {
            println!("Pin {} changed to {:?}", pin, level);
            // TODO send to MQTT module

            true
        },

        gpio_watcher::GpioPollResult::Stopped => false
    } {}
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
