# HA-GPIO-MQTT

A small service which sends GPIO events to an MQTT service to create binary sensor entities in Home Assistant.
Created as an alternative to the Remote GPIO integration, which at the time of writing always fails for me after the Pi restarts and does not recover until Home Assistant is restarted.

## Compiling
Install [Rust](https://www.rust-lang.org/) for your operating system of choice or directly on the Raspberry Pi. Make sure to add support for the target ```armv7-unknown-linux-gnueabihf``` if you're not using the Pi itself. Supposedly you can do this via Rustup but don't ask me how as I use NixOS and the rust-overlay to specify additional targets.

Build using ```cargo build --release --target armv7-unknown-linux-gnueabihf``` (target is only required when cross-compiling, not if you're compiling directly on the Raspberry Pi).

## Configuring
TODO

## Installing as a service
TODO