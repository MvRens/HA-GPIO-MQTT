use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use rppal::gpio::{Gpio, InputPin};

use crate::config::{Config, ConfigGpioPull};


pub struct GpioWatcher
{
    worker: Option<thread::JoinHandle<()>>,
    control_sender: Sender<GpioControlMessage>,
    status_receiver: Receiver<GpioStatusMessage>
}


pub enum GpioPollResult
{
    None,
    PinChanged(u8, GpioPinLevel),
    Stopped
}


enum GpioControlMessage
{
    Stop
}


enum GpioStatusMessage
{
    PinChanged(u8, GpioPinLevel)
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
    pub fn start(config: &Config) -> Self
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

        let (control_sender, control_receiver) = channel();
        let (status_sender, status_receiver) = channel();


        let worker = Some(thread::spawn(move ||  
        {
            let Ok(gpio) = Gpio::new() else 
            {
                log::error!("Failed to initialize GPIO module");
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
                            Err(e) => 
                            {
                                log::error!("Failed to set interrupt for pin {}: {}", *pin, e);
                                GpioPinState::Error()
                            }
                        }
                    },

                    Err(e) => 
                    {
                        log::error!("Failed to acquire pin {}: {}", *pin, e);
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


            // Output initial state as change events
            for (pin, info) in pin_map.iter()
            {
                let GpioPinState::Input(input_pin) = &info.state else { continue }; 
                let level = input_pin.read();
                
                send_pin_changed(&status_sender, *pin, level, &info);
            }


            #[allow(while_true)]
            while true
            {
                match control_receiver.try_recv()
                {
                    Ok(GpioControlMessage::Stop) |
                    Err(TryRecvError::Disconnected) =>
                    {
                        break;
                    },

                    Err(TryRecvError::Empty) => {}
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
                                send_pin_changed(&status_sender, number, level, pin_info);
                            }
                        },

                        None => break
                    }
                }
            }
        }));

        Self
        {
            worker,
            control_sender,
            status_receiver
        }
    }


    pub fn poll(&self) -> GpioPollResult
    {
        match self.status_receiver.try_recv()
        {
            Ok(GpioStatusMessage::PinChanged(pin, level)) => GpioPollResult::PinChanged(pin, level),

            Err(TryRecvError::Disconnected) =>
            {
                GpioPollResult::Stopped
            },

            _ => GpioPollResult::None
        }
    }


    pub fn stop(&mut self)
    {
        self.control_sender.send(GpioControlMessage::Stop).unwrap_or_default();
        self.worker.take().map(thread::JoinHandle::join);
    }
}


fn get_pin_level(level: rppal::gpio::Level, inverted: bool) -> GpioPinLevel
{
    // TODO is this required, or does the pullup already handle this?? seems that way.
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


fn send_pin_changed(status_sender: &Sender<GpioStatusMessage>, pin: u8, level: rppal::gpio::Level, pin_info: &GpioPinInfo)
{
    let level = get_pin_level(level, pin_info.inverted);

    if let Err(e) = status_sender.send(GpioStatusMessage::PinChanged(pin, level))
    {
        log::error!("Failed to send status message: {}", e);
    }
}