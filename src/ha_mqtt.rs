use std::collections::HashMap;
use std::thread;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::time::Duration;

use rumqttc::{Client, MqttOptions};
use serde::Serialize;

use crate::config::Config;


pub struct HaMqtt
{
    worker: Option<thread::JoinHandle<()>>,
    control_sender: Sender<HaMqttControlMessage>
}


enum HaMqttControlMessage
{
    Stop,
    PinChanged(u8, bool)
}


struct HaMqttPinInfo
{
    entity_id: String,
    name: String
}


#[derive(Serialize, Debug)]
struct HaMqttConfigPayload
{
    name: Option<String>,
    device_class: String,
    state_topic: String,
    unique_id: String,
    device: HaMqttConfigDevicePayload
}


#[derive(Serialize, Debug)]
struct HaMqttConfigDevicePayload
{
    identifiers: Vec<String>,
    name: String
}


impl HaMqtt
{
    pub fn start(config: &Config) -> Self
    {
        let pin_map: HashMap<u8, HaMqttPinInfo> = config.gpio.iter()
            .map(|io| (io.pin, HaMqttPinInfo 
            {
                entity_id: io.entity_id.clone(),
                name: io.name.clone()
            }))
            .collect();

        let mqtt_options = MqttOptions::new(config.client_id.clone(), config.mqtt.hostname.clone(), config.mqtt.port)
            .set_credentials(config.mqtt.username.clone(), config.mqtt.password.clone())
            .set_keep_alive(Duration::from_secs(5))
            .to_owned();
        
        let (control_sender, control_receiver) = channel();
        let discovery_prefix = config.discovery_prefix.clone();

        let worker = Some(thread::spawn(move ||  
        {
            let (client, mut connection) = Client::new(mqtt_options, 10);

            thread::spawn(move ||
            {
                for (_, notification) in connection.iter().enumerate()
                {
                    log::debug!("Notification: {:?}", notification);
                }
            });


            // Send discovery messages to initialize all pins
            // TODO also do this on reconnect / if HA's Birth message is received
            for (pin, info) in pin_map.iter()
            {
                let topic = format!("{}/binary_sensor/{}/config", discovery_prefix, info.entity_id);
                let payload = HaMqttConfigPayload
                {
                    name: None,
                    device_class: String::from("motion"),
                    state_topic: String::from(format!("{}/binary_sensor/{}/state", discovery_prefix, info.entity_id)),
                    unique_id: String::from(info.entity_id),
                    device: HaMqttConfigDevicePayload
                    {
                        identifiers = 
                    }
                };

                if let Ok(payload_json) = serde_json::to_string(&payload)
                {
                    if let Err(e) = client.publish(topic, rumqttc::QoS::AtLeastOnce, false, payload_json)
                    {
                        log::error!("Failed to publish discovery message for pin {}: {}", pin, e);
                    };
                }
            }


            #[allow(while_true)]
            while let Ok(m) = control_receiver.recv()
            {
                match m
                {
                    HaMqttControlMessage::PinChanged(pin, on) =>
                    {
                        if let Some(pin_info) = pin_map.get(&pin)
                        {
                            let topic = format!("{}/binary_sensor/{}/config", discovery_prefix, pin_info.entity_id);
                            let payload = "TODO";
    
                            if let Err(e) = client.publish(topic, rumqttc::QoS::AtLeastOnce, true, payload)
                            {
                                log::error!("Failed to publish message: {}", e);
                            }    
                        }
                    },

                    HaMqttControlMessage::Stop =>
                        break
                }
            }

            let _ = client.disconnect();
        }));

        Self
        {
            worker,
            control_sender
        }
    }


    pub fn pin_changed(&self, pin: u8, on: bool)
    {
        if let Err(e) = self.control_sender.send(HaMqttControlMessage::PinChanged(pin, on))
        {
            log::error!("Failed to send message to worker: {}", e);
        }
    }


    pub fn stop(&mut self)
    {
        self.control_sender.send(HaMqttControlMessage::Stop).unwrap_or_default();
        self.worker.take().map(thread::JoinHandle::join);
    }
}