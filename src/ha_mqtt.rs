use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
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
    PinChanged(u8, bool),
    ResendConfig
}


struct HaMqttPinInfo
{
    entity_id: String,
    name: String,
    device_class: Option<String>
}


#[derive(Serialize, Debug)]
struct HaMqttConfigPayload
{
    name: String,
    device_class: Option<String>,
    state_topic: String,
    unique_id: String
}


impl HaMqtt
{
    pub fn start(config: &Config) -> Self
    {
        let pin_map: HashMap<u8, HaMqttPinInfo> = config.gpio.iter()
            .map(|io| (io.pin, HaMqttPinInfo 
            {
                entity_id: io.entity_id.clone(),
                name: io.name.clone(),
                device_class: io.device_class.clone()
            }))
            .collect();

        let mqtt_options = MqttOptions::new(config.mqtt.client_id.clone(), config.mqtt.hostname.clone(), config.mqtt.port)
            .set_credentials(config.mqtt.username.clone(), config.mqtt.password.clone())
            .set_keep_alive(Duration::from_secs(5))
            .to_owned();
        
        let (control_sender, control_receiver) = channel();
        let discovery_prefix = config.homeassistant.discovery_prefix.clone();
        let birth_topic = config.homeassistant.birth_topic.clone();
        let state_prefix = config.homeassistant.state_prefix.clone();
        let device_name = config.homeassistant.device_name.clone();

        let control_sender_eventloop = control_sender.clone();

        let worker = Some(thread::spawn(move ||  
        {
            let (client, mut connection) = Client::new(mqtt_options, 10);
            let is_reconnect = Arc::new(AtomicBool::new(false));

            if let Err(e) = client.subscribe(birth_topic, QoS::AtLeastOnce)
            {
                log::error!("Failed to subscribe to birth topic: {}", e);
            }


            thread::spawn(move ||
            {
                for notification in connection.iter().flatten()
                {
                    log::debug!("MQTT notification: {:?}", notification);

                    match notification
                    {
                        Event::Incoming(Incoming::Publish(packet)) =>
                        {
                            let payload_bytes: &[u8] = &packet.payload;

                            let Ok(msg) = std::str::from_utf8(payload_bytes) else { continue };
        
                            if msg == "online"
                            {                 
                                log::info!("HomeAssistant sent birth message, resending configuration");
                                let _ = control_sender_eventloop.send(HaMqttControlMessage::ResendConfig);
                            }
                        },

                        Event::Incoming(Incoming::ConnAck(_)) =>
                        {
                            if is_reconnect.swap(true, Ordering::Relaxed)
                            {
                                log::info!("Connection to MQTT re-established, resending configuration");
                                let _ = control_sender_eventloop.send(HaMqttControlMessage::ResendConfig);
                            }
                        },

                        _ => {}
                    }
                }
            });


            send_config_messages(&client, &pin_map, &discovery_prefix, &state_prefix, &device_name);


            #[allow(while_true)]
            while let Ok(m) = control_receiver.recv()
            {
                match m
                {
                    HaMqttControlMessage::PinChanged(pin, on) =>
                    {
                        if let Some(pin_info) = pin_map.get(&pin)
                        {
                            let topic = format!("{}/binary_sensor/{}/{}/state", state_prefix, device_name, pin_info.entity_id);
                            let payload = match on
                            {
                                true => "ON",
                                false => "OFF"
                            };
    
                            if let Err(e) = client.publish(topic, rumqttc::QoS::AtLeastOnce, false, payload)
                            {
                                log::error!("Failed to publish status message for pin {}: {}", pin, e);
                            }
                        }
                    },

                    HaMqttControlMessage::ResendConfig =>
                        send_config_messages(&client, &pin_map, &discovery_prefix, &state_prefix, &device_name),

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


fn send_config_messages(client: &Client, pin_map: &HashMap<u8, HaMqttPinInfo>, discovery_prefix: &str, state_prefix: &str, device_name: &str)
{
    // Send discovery messages to initialize all pins
    for (pin, info) in pin_map.iter()
    {
        let topic = format!("{}/binary_sensor/{}/{}/config", discovery_prefix, device_name, info.entity_id);
        let payload = HaMqttConfigPayload
        {
            name: info.name.clone(),
            device_class: info.device_class.clone(),
            state_topic: format!("{}/binary_sensor/{}/{}/state", state_prefix, device_name, info.entity_id),
            unique_id: info.entity_id.clone()
        };

        if let Ok(payload_json) = serde_json::to_string(&payload)
        {
            if let Err(e) = client.publish(topic, rumqttc::QoS::AtLeastOnce, false, payload_json)
            {
                log::error!("Failed to publish discovery message for pin {}: {}", pin, e);
            };
        }
    }
}