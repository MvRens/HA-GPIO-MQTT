use std::{sync::mpsc::{channel, Receiver, Sender}, thread::{self, JoinHandle}};

use rppal::gpio::{Error, Gpio, InputPin};

use crate::config::Config;


pub struct GpioHandler
{
    sender: Sender<GpioControlMessage>,
    receiver: Receiver<GpioStatusMessage>,
    worker: JoinHandle<()>
}


enum GpioControlMessage
{
}


enum GpioStatusMessage
{
}


struct GpioPin
{
    number: u8,
    inverted: bool,
    pin: InputPin
}


impl GpioHandler
{
    pub fn start(config: &Config) -> Result<Self, Error>
    {
        let gpio = Gpio::new()?;
        let pins: Vec<GpioPin> = config.gpio.iter()
            .map(|io| GpioPin 
            {
                number: io.pin,
                inverted: io.inverted,
                // TODO get rid of the unwrap 
                // TODO support for pull down / up
                pin: gpio.get(io.pin).unwrap().into_input()
            })
            .collect();

        let (control_sender, control_receiver) = channel();
        let (status_sender, status_receiver) = channel();

        let worker = thread::spawn(move ||  
        {
            /*
            // Retrieve the GPIO pin and configure it as an output.
            let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output_low();

            // Wait for an incoming message. Loop until a None is received.
            while let Some(count) = receiver.recv().unwrap() {
                println!("Blinking the LED {} times.", count);
                for _ in 0u8..count {
                    pin.set_high();
                    thread::sleep(Duration::from_millis(250));
                    pin.set_low();
                    thread::sleep(Duration::from_millis(250));
                }
            }

            Ok(())
            */
        });


        Ok(Self
        {
            sender: control_sender,
            receiver: status_receiver,
            worker
        })
    }
}