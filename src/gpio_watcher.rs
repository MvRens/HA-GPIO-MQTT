use std::{collections::HashMap, sync::mpsc::{channel, Sender, TryRecvError}, thread::{self, JoinHandle}, time::Duration};

use rppal::gpio::{Gpio, InputPin};

use crate::config::{Config, ConfigGpioPull};


pub struct GpioWatcher
{
    worker: Option<JoinHandle<()>>,
    worker_sender: Sender<GpioControlMessage>
}


enum GpioControlMessage
{
    Stop
}


struct GpioPinInfo
{
    inverted: bool,
    pull: Option<GpioPinPull>,
    state: GpioPinState
}


enum GpioPinPull
{
    Up,
    Down
}


enum GpioPinState
{
    Init,
    Input(InputPin),
    Error()
}


#[derive(Debug)]
pub enum GpioPinLevel
{
    Low,
    High
}


impl GpioWatcher
{
    pub fn start(config: &Config, on_pin_changed: fn(u8, GpioPinLevel)) -> Self
    {
        let mut pin_map: HashMap<u8, GpioPinInfo> = config.gpio.iter()
            .map(|io| (io.pin, GpioPinInfo 
            {
                inverted: io.inverted,
                state: GpioPinState::Init,
                pull: match io.pull
                {
                    ConfigGpioPull::None => None,
                    ConfigGpioPull::Up => Some(GpioPinPull::Up),
                    ConfigGpioPull::Down => Some(GpioPinPull::Down)
                }
            }))
            .collect();

        let (worker_sender, worker_receiver) = channel();

        let worker = thread::spawn(move ||  
        {
            let Ok(gpio) = Gpio::new() else 
            {
                // TODO logging
                return;
            };


            // Initialize pins
            for (pin, info) in pin_map.iter_mut()
            {
                info.state = match gpio.get(*pin) 
                {
                    Ok(v) => 
                    {
                        let mut input_pin = match info.pull
                        {
                            None => v.into_input(),
                            Some(GpioPinPull::Up) => v.into_input_pullup(),
                            Some(GpioPinPull::Down) => v.into_input_pulldown()
                        };

                        match input_pin.set_interrupt(rppal::gpio::Trigger::Both)
                        {
                            Ok(_) => GpioPinState::Input(input_pin),
                            Err(_) => 
                            {
                                // TODO logging
                                GpioPinState::Error()
                            }
                        }
                    },

                    Err(_) => 
                    {
                        // TODO logging
                        GpioPinState::Error()
                    }
                };
            }

            let pins = pin_map
                .values()
                .filter_map(|i| match &i.state
                {
                    GpioPinState::Input(pin) => Some(pin),
                    _ => None
                })
                .collect::<Vec<&InputPin>>();


            #[allow(while_true)]
            while true
            {
                match worker_receiver.try_recv()
                {
                    Ok(GpioControlMessage::Stop) |
                    Err(TryRecvError::Disconnected) =>
                    {
                        break;
                    },

                    _ => {}
                }


                while let Ok(r) = gpio.poll_interrupts(&pins, false, Some(Duration::from_millis(10)))
                {
                    match r
                    {
                        Some((pin, level)) =>
                        {
                            let number = pin.pin();

                            if let Some(pin_info) = pin_map.get(&number)
                            {
                                let level = get_pin_level(level, pin_info.inverted);
                                on_pin_changed(number, level);
                            }
                        },

                        None => break
                    }
                }
            }
        });


        Self
        {
            worker_sender,
            worker: Some(worker)
        }
    }


    pub fn stop(&mut self)
    {
        self.worker_sender.send(GpioControlMessage::Stop).unwrap_or_default();
        self.worker.take().map(JoinHandle::join);
    }
}


fn get_pin_level(level: rppal::gpio::Level, inverted: bool) -> GpioPinLevel
{
    if inverted
    {
        match level
        {
            rppal::gpio::Level::Low => GpioPinLevel::High,
            rppal::gpio::Level::High => GpioPinLevel::Low
        }
    }
    else
    {
        match level
        {
            rppal::gpio::Level::Low => GpioPinLevel::Low,
            rppal::gpio::Level::High => GpioPinLevel::High
        }        
    }
}