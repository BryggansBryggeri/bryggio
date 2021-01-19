# BryggIO

BRYGGANS BRYGGERI's very own brewery control software.

Currently under heavy developement.
The goal is to develop a stand-alone pub-sub backend with which any client can communicate and thereby control the brewery hardware.

## Philosophy

Having started our brewery career with first a horrible Python loop and then the much nicer [Craftbeer Pi](http://web.craftbeerpi.com/)
we knew we always wanted to write our own brewery software.

We are ever grateful for Craftbeer Pi, which has helped us brew a lot of beer,
but there were a few things we did not like with it:

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
  **Note:** We are currently in pre-release (<1.0, <0.1.0 to be perfectly honest) and during that period wild changes can and will occur.
- BryggIO is, and will remain, free and open source. We will keep backend and frontend separated so that any frontend can be used.

### Control

Standard is to use a simple PID controller. This is slightly nihilistic given the almost complete model information one usually has when dealing with a simple brewery.
The arguments in favour of PID control are: ignorance, efficiency, laziness; what if we disregard some information? The system is slow and simple and it works fine.

BryggIO takes the opposite approach: Given a simple and slow system, we have the possibility to have a complicated control. Furthermore, a slow system demands the best possible control in order to save precious time and energy.

Due to the inherent inertia in the objective (heating a lot of water), we can have quite long time between signal updates, providing time to calculate a nearly optimal control sequence.

## Installation

Before the first release we will not publish any binaries, see [Install from source](#install-from-source)

### Build from source

- Install rust, cargo and cargo-make from [here](https://www.rust-lang.org/tools/install).
  Rust and cargo are provided by official distributions, cargo-make can subsequently be installed with

  ```bash
  cargo install cargo-make
  ```

- Build targets `bryggio-supervisor`, `bryggio-cli` and `nats-server` from source.
  The latest released version `nats-server` does not yet support web-sockets, so it needs to be built from the master branch.
  This step obviously requires installation of [golang](https://golang.org/).
  Hopefully this feature will [soon](https://nats.io/about/) be on the stable release and a simple download will suffice.
  Until then, the NATS server [repo](https://github.com/nats-io/nats-server) is included as a submodule in the BryggIO repo.

  ```bash
  git clone --recurse-submodules git@github.com:BryggansBryggeri/bryggio.git bryggio
  cd bryggio
  cargo make --no-workspace build
  cargo run --bin bryggio-supervisor -- <path_to_bryggio_config_file>
  ```

## Configuration

- **BryggIO config:** JSON file which specifies general settings, and importantly **the path to the `nats-server` binary and corresponding config file**.
  See `sample-bryggio.json` for an example.
- **NATS config:** particular YAML config file for the `nats-server`.
  See `sample-nats-config.yaml` for an example.
  This will be integrated into the general BryggIO config.

Check out the sample configs in this repo for usage.

The supervisor, starts up a `nats-server` in a separate process and then runs a supervisor pub sub client which,
listening to special command subjects, starts and stops other clients like sensors, actors and controllers.

## Run

There are two ways to run the supervisor:

```bash
# This will recompile if there are code changes
cargo run --bin bryggio-supervisor -- run <path_to_bryggio_config_file>
# ... while this will simply run the executable created in the build step.
./target/<profile>/bryggio-supervisor run <path_to_bryggio_config_file>
```

## Build for and run on rbpi

Build for rbpi needs an arm-compatible rust toolchain. Install with

```bash
rustup target add armv7-unknown-linux-gnueabihf
```

and build the required executables

```bash
# In the bryggio repo root
cargo make rbpi-build
```

TODO: Add rbpi host and path as env. variables and make them available to the `cargo make rbpi-install` command.

Move the resulting executables
(`target/armv7-unknown-linux-gnueabihf/<build-mode>/bryggio-supervisor`) and
(`target/rbpi-nats-server`)
to the rbpi, as well as the config files listed in [Configuration](#configuration)

On the rbpi run:

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
