# HA-GPIO-MQTT

A small service which sends GPIO events to an MQTT service to create binary sensor entities in Home Assistant.
Created as an alternative to the Remote GPIO integration, which at the time of writing always fails for me after the Pi restarts and does not recover until Home Assistant is restarted.

## Compiling
Install [Rust](https://www.rust-lang.org/) for your operating system of choice or directly on the Raspberry Pi. Make sure to add support for the target ```armv7-unknown-linux-musleabihf``` if you're not using the Pi itself. Supposedly you can do this via Rustup but don't ask me how as I use NixOS and the rust-overlay to specify additional targets. You likely also need the ARM GNU toolchain somewhere in your PATH.

Build using ```cargo build --release``` (target is set for Raspberry Pi by default).

## Configuring
TODO

## Installing as a service
TODO