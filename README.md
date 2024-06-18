# HA-GPIO-MQTT

A small service which sends GPIO events to an MQTT service to create binary sensor entities in Home Assistant.
Created as an alternative to the Remote GPIO integration, which at the time of writing always fails for me after the Pi restarts and does not recover until Home Assistant is restarted.

## Compiling
I tried to get it to cross-compile but was not yet succesful. I recommend installing [Rust](https://www.rust-lang.org/) directly on the Raspberry Pi and running ```cargo build --release``` to build the executable ```target/release/ha-gpio-mqtt```.

### Cross-compiling
After installing rust add support for the target ```armv7-unknown-linux-musleabihf```. The required rustup command shows up if you try to build for this target and don't have it yet. I use NixOS and the rust-overlay to install it. You also need the ARM GNU toolchain somewhere in your PATH or open up a nix-shell which will use the provided shell.nix.

1. Uncomment the target configuration in ```.config/config.toml``` to set the default build target and correct linker and compiler flags.
2. I used the 'musl' version of the standard libraries to try and get a statically linked executable, as it would give the error 'No such file or directory'.
3. I had to use ```patchelf --set-interpreter /usr/lib/ld-linux-armhf.so.3``` because Rust still created a dynamically linked executable that wouldn't load. So perhaps step 2 wasn't actually the issue and the musl version is causing the issue in step 4, but I want to get the project running before attempting to fix this.
4. Using the above changes it would run on the Raspberry Pi, but after loading the config file it would segfault. This is where I gave up and simply compiled on the Raspberry Pi itself, which worked.

I'm still interested in getting cross-compilation to work properly, so it's easier to get projects like these in a CI build. If anyone knows what I did wrong, let me know!

## Configuring
TODO

## Installing as a service
TODO