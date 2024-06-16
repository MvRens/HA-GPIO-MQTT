use std::{sync::mpsc::{channel, Receiver, Sender, TryRecvError}, thread::{self, JoinHandle}};

use rppal::gpio::{Error, Gpio, InputPin};

use crate::config::{Config, ConfigGpioPull};


pub struct GpioHandler
{
    sender: Sender<GpioControlMessage>,
    receiver: Receiver<GpioStatusMessage>,
    worker: Option<JoinHandle<()>>
}


enum GpioControlMessage
{
    Stop
}


enum GpioStatusMessage
{
}


struct GpioPin
{
    number: u8,
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
    Error(Error)
}


impl GpioHandler
{
    pub fn start(config: &Config) -> Self
    {
        let pins: Vec<GpioPin> = config.gpio.iter()
            .map(|io| GpioPin 
            {
                number: io.pin,
                inverted: io.inverted,
                state: GpioPinState::Init,
                pull: match io.pull
                {
                    ConfigGpioPull::None => None,
                    ConfigGpioPull::Up => Some(GpioPinPull::Up),
                    ConfigGpioPull::Down => Some(GpioPinPull::Down)
                }
            })
            .collect();

        let (control_sender, control_receiver) = channel();
        let (status_sender, status_receiver) = channel();

        let worker = thread::spawn(move ||  
        {
            let Ok(gpio) = Gpio::new() else 
            {
                // TODO logging
                return;
            };

                // TODO support for pull down / up
//                pin: gpio.get(io.pin).unwrap().into_input()


            // Initialize pins
            for pin in pins
            {
                pin.state = match gpio.get(pin.number) 
                {
                    Ok(v) => GpioPinState::Input(match pin.pull
                    {
                        None => v.into_input(),
                        Some(GpioPinPull::Up) => v.into_input_pullup(),
                        Some(GpioPinPull::Down) => v.into_input_pulldown()
                    }),
                    Err(e) => 
                    {
                        // TODO logging
                        GpioPinState::Error(e)
                    }
                };

                if let GpioPinState::Input(v) = pin.state
                {
                    v.set_interrupt(rppal::gpio::Trigger::Both);
                };
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

                    _ => {}
                }


                while let Some(i) = gpio.poll_interrupts(pins, false, 10)
                {

                }
            }
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
            worker: Some(worker)
        })
    }


    pub fn stop(&mut self)
    {
        self.sender.send(GpioControlMessage::Stop).unwrap_or_default();
        self.worker.take().map(JoinHandle::join);
    }
}