# BryggIO

BRYGGANS BRYGGERI's very own brewery control software.

Currently under heavy developement.
The goal is to develop a stand-alone Pub-sub backend with which any client can communicate with and thereby control the brewery hardware.

## Philosophy

Having started our brewery career with first a horrible Python loop and then the much nicer [Craftbeer Pi](http://web.craftbeerpi.com/)
we knew we always wanted to write our own brewery software.

Although we are ever grateful for Craftbeer Pi, which has helped us brew a lot of beer,
there were a few things we did not like with it:

- Reliability, sometimes sensors or plugins would stop working and a restart was the only fix.
- There were a lot of versions in use at the same time.
- Modifying the software, especially the frontend was a bit obscure.

Most of all, we always had the goal of making our own brewery software so do take this mild criticism of CbPi as a
rationalisation for us to spend a lot of time on BryggIO.

To remedy these projects BryggIO is:

- Implemented in rust --- a strictly typed, compiled and safe language (no unsafe code is used in the core software) ---
  BryggIO is very reliable. The compiler checks many edge cases and crashing the software is extremely difficult.
  As a bonus, it also makes BryggIO very fast and memory efficient.
- We aim to be very stable and use semantic versioning to communicate changes' impact.
  *Note:* We are currently in pre-release (<1.0, <0.1.0 to be perfectly honest) and during that period wild changes can and will occur.
- BryggIO is, and will remain, free and open source. We will keep backend and frontend separated so that any frontend can be used.

### Control

Standard is to use a simple PID controller. This is slightly nihilistic given the almost complete model information one usually has when dealing with a simple brewery.
The arguments in favour of PID control are: ignorance, efficiency, laziness; what if we disregard some information? The system is slow and simple and it works fine.

BryggIO takes the opposite approach: Given a simple and slow system, we have the possibility to have a complicated control. Furthermore, a slow system demands the best possible control in order to save precious time and energy.

Due to the inherent inertia in the objective (heating a lot of water), we can have quite long time between signal updates, providing time to calculate a nearly optimal control sequence.

## Installation

  Before the first release we will not publish any binaries, see [Install from source](#install-from-source)

## Install from source

 - Install rust from [here](https://www.rust-lang.org/tools/install).

 - Install `nats-server`.
   The latest released version does not yet support web-sockets, so it needs to be built from the master branch.
   This step obviously requires installation of golang, hopefully this feature will soon be on the stable release and a simple download will suffice.
   ```bash
   git clone --branch=master https://github.com/nats-io/nats-server.git nats-server
   cd nats-server
   go build
   ```
 - Configuration is currently split into
    - *`bryggio` config:* TOML file which specifies general settings, and importantly *the path to the `nats-server` binary and config file*.
    - *`nats`* particular YAML config file for the `nats-server`.

   Check out the sample configs in this repo for usage.

 - Clone the `bryggio` repo and run the binary `bryggio-supervisor`.
   ```bash
   git clone git@github.com:BryggansBryggeri/bryggio.git bryggio
   cd bryggio
   git checkout pub_sub # Pub sub version not yet merged.
   cargo run --bin bryggio-supervisor -- <path_to_bryggio_config_file>
   ```

   The supervisor, starts up a `nats-server` in a separate process and then runs a supervisor pub sub client which,
   listening to special command subjects, starts and stops other clients like sensors, actors and controllers.

## Run on Rbpi

Build for rbpi needs

- arm-compatible rust toolchain installed

```bash
cargo make rbpi-build
```

Move the resulting binary (`target/armv7-unknown-linux-gnueabihf/<build-mode>/bryggio-supervisor`) to the rbpi.

Also needed: A `bryggio` TOML config file.

On the rbpi, currently the config files need to be in the same directory as the binary, then:

```bash
sudo ./bryggio-supervisor <path_to_bryggio_config_file>
# E.g.:
# sudo ./bryggio-supervisor sample-bryggio.toml
```
`sudo` is required for gpio manipulation.

### Literature
- Energy efficiency: (https://greenlab.di.uminho.pt/wp-content/uploads/2017/10/sleFinal.pdf)
- (http://www.iiisci.org/journal/CV$/sci/pdfs/ZA191KB18.pdf)
- (http://kchbi.chtf.stuba.sk/upload_new/file/Miro/Proc%21problemy%20odovzdane%20zadania/%C5%A0uhaj/Predictive%20modelling%20of%20brewing%20fermentation%20from%20knowledge-based%20to%20black-box%20models.pdf)

(https://scholarworks.uni.edu/cgi/viewcontent.cgi?article=1661&context=etd)
