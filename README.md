# BryggIO

BRYGGANS BRYGGERI's very own brewery control software.

Currently under heavy developement.
The goal is to develop a stand-alone pub-sub backend with which any client can communicate and thereby control the brewery hardware.

## Project overview

*In this section we give a high-level overview of BryggIO's architecture.
It is intended to introduce new potential contributors to the application and underlying design choices.*

### Structure
The repo contains four crates (a crate is a Rust library or executable):

- `bryggio_core`: The main library where most functionality is implemented.
- `bryggio_supervisor`: The main executable that is running during the brewing process. It is a very small crate, that starts up and runs a BryggIO supervisor client,
  which controls the brewing process.
- `bryggio_sensor_box`: A separate executable that controls an external sensor interface. Essentially a smaller version of `bryggio_supervisor`.
- `bryggio_cli`: A command line tool which handles installation and allows for debugging communication with the `bryggio_supervisor`.

### Short note on language choice

BryggIO is written in a language called [Rust](https://www.rust-lang.org/) I don't really find the choice of language particularly interesting but Rust is still a rather niche language, prompting a brief motivation for choosing Rust.
The language candidates I considered were: Python, C, C++ and Rust.
To me, Rust have some properties making it the language, among the ones I know, best suited for an application like this.

These are:

 - **Robustness:** Rust is often described as safe. BryggIO will initially be run on local networks, making safety less important.
   However, the same mechanisms that make Rust safe, also make it robust.
   Our goal is to produce an application that just works, and keeps on working. Rusts expressive type system, pedantic compiler and lack of exceptions is a perfect match for BryggIO.
   It is hard to make the code compile, but when it finally does, you can trust it to a large extent.
   The same thing does not apply to the other languages considered.

 - **Modern dependency management:** Rust comes with a modern package manager called "Cargo".
   It makes it easy to compile the source code and importantly, to include third-party dependencies.
   Python has similar functionality, while C and C++ do not.

 - **Low-level:** The target platform for BryggIO is a Raspberry Pi (rbpi), which in this context is hardly embedded, but in the future we will likely end up with some clients running on microcontrollers.
   This rules out Python. C and C++ have better embedded support but Rust is good enough to not disqualify it based on this property.
   Furthermore, though not embedded, the rbpi is resource constrained, making a language without a runtime preferable.

### Components

The automated brewing process can be quite simple.
The foundation of it is temperature control. We observe temperature measurements and control the output of a heating element to follow a target temperature.

In BryggIO, we have modelled this process with three component types: sensors, actors and controllers.
A sensor is sort of a passive, independent component. Intermittently, it emits a measurement.
An actor is something that we can control, like a heating element, a pump, et c; something that can be turned on and off.
Both sensors and actors are associated with real hardware.
A controller is an abstract component. It listens for measurement from a sensor, computes a signal that it sends to an actor, in order to reach a set target.

![Component structure](assets/component_structure.svg)

The component types are represented in Rust's type system as "traits", which are like "interfaces" in other languages.
A trait defines common functionality that a set of concrete types share.
For example, the `Sensor` trait, defined in `bryggio_core::sensor`, looks like this:
```rust
pub trait Sensor: Send {
    fn get_measurement(&mut self) -> Result<f32, SensorError>;
    fn get_id(&self) -> String;
}
```
We make a concrete type become a sensor type by implementing the `Sensor` trait, e.g. for a dummy sensor that we use for prototyping:

```rust
pub struct DummySensor {
    pub id: String,
    latest_value: f32,
    delay: Duration,
    rng: Normal<f32>,
}

impl Sensor for DummySensor {
    fn get_measurement(&mut self) -> Result<f32, SensorError> {
        let measurement = self.latest_value + self.rng.sample(&mut thread_rng()) / 10.0;
        self.latest_value = measurement;
        sleep(self.delay);
        Ok(measurement)
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
```
NB: Rust does not have object-oriented inheritance. We don't have a "sensor base class/type" but rather a set of functions that each sensor type must provide.

Using the trait system enables us to define types for every sensor that we want to use and still have a common interface for them.
There are corresponding traits for actors and controllers.

We have imposed an invariant that each controller has a single actor and a single sensor.
This may seem restrictive but remember that, for instance, what constitutes a sensor has a wide interpretation.
Let's say we want to use two thermometers instead of one, and control their average temp.
Well, then we'll simply define a new concrete sensor type "AverageSensor" which internally reads the two temp's,
computes the average and sends that "meta-measurement" to the controller.
That is, we get the more complex functionality that we want but still uphold the invariant.

Another restriction is that an actor is always associated with a controller.
That is, even though you could imagine two modes of actor operation: an automated actor, run by a controller and a manual actor that is turned on/off, the API does not allow for the latter.
This is for safety, the actors control the high-power hardware, so it is reasonable to only have one way to do it.
For the kind of simple manual control, we have a manual controller that acts as a middle man, yet still conforms to the design pattern.

### Publish-subscribe pattern

The communication between components such as sensors and controllers we use a publish-subscribe pattern, which you can read about [here](https://ably.com/topic/pub-sub).
The gist of it is that components don't communicate directly with each other.
Instead, components act like "clients", whcih communicate via a central server.
A client can publish messages to the server, messages which are tagged with a "subject".
Clients receive messages by subscribing to subjects.
In this centralised way, clients can communicate without really having knowledge of each other.

![Pub-sub pattern](assets/pub_sub.svg)

For instance, when a sensor makes a measurement it simply publishes a message with subject "temp\_sensor\_1.measurement" containing the measurement.
Any other interested client -- i.e. that subscribes to this topic -- receives this message. This can be a controller client, which uses it to publish a new control signal,
or even the web UI that merely displays the value.

The pub-sub pattern makes client more independent, and new clients can be added without having to update the existing set of clients.
Furthermore, the communication is language agnostic, one could for instance write a, say, twitter bot in Python that listens to the server and makes updates about the brewing process.

The pattern is also encoded in Rust's type system. All components -- now clients -- implement the `PubSubClient` trait in `bryggio_core::pub_sub`.
We take further advantage of Rust's generic type system by providing generic implementations for the different component types.
In Rust it is possible to make a generic implementation for all types that implements a second trait.
This allows us to reduce code repetition by, for instance, implementing the `PubSubClient` trait for each type that implements the `Sensor` trait.

In short, pub-sub is well-suited for this application. With that said, there are some communication that require more direct communication, i.e. with acknowledgements.
NATS provide this out-of-the-box, with the [request-reply](https://docs.nats.io/nats-concepts/reqreply) feature.
Using that a message can be explicitly responded to, this is vital for messages that can't be lost (like shutting down some high-power hardware).

For the actual pub-sub system we use a protocol called [NATS](https://nats.io/). NATS is really just a protocol but it also comes with a server implementation that we use in BryggIO.
A pub-sub server is a complex piece of software so it is nice to use an existing solution.
Other alternatives exist, like [MQTT](https://mqtt.org/), but thus far we are quite happy with NATS.

### The supervisor client

Since we use the pub-sub pattern we could in principle have one executable for every client, and run them in separate process.
This would not be convenient however, so instead we have a special "Supervisor" client which is run as an executable.
The supervisor is responsible for starting and monitoring all our basic clients, like sensors, actors and controllers.
Via pub-sub messages we can also shut-down and start new clients during the brewing process.

### User interface (UI)

BryggIO intentionally does not come with a UI. A clear design goal is to have a full decoupling between frontend and backend.
The beauty of pub-sub is that any external application, like a UI, with a working NATS client can participate with any part of the brewing process.
We are developing a web-based UI, [BryggUI](https://github.com/BryggansBryggeri/bryggui), but whereas BryggIO is general and configurable,
the web UI is tailored to a specific brewery setup (our own).

## Issues/Road map

- **Asynchronicity:** This application is async. in nature; sensors publish measurements at semi-random interval, triggering controllers to compute a new signal, which actors react to.
  The entire design would be much more intuitive if we managed to make it async.
  We have done some minor work exploring async. but much more remain to be done here, see [Tracking issue: Async](https://github.com/BryggansBryggeri/bryggio/issues/55).
- **Permanent data logging:** Currently, no data is stored. We will start out with some simple, writing to file and from there explore proper database options.
- **BryggIO protocol:** The system of subjects that structure the communication have grown organically.
  Some work should be dedicated to create a more principled protocol.
- **1.0 version:** For a more detailed version, see the [Github project](https://github.com/BryggansBryggeri/bryggio/projects/2).

## Installation

Before the first release we will not publish any binaries, see [Build from source](#build-from-source)

### Build from source

- Install rust, cargo and cargo-make from [here](https://www.rust-lang.org/tools/install).
  Rust and cargo are provided by official distributions, cargo-make can subsequently be installed with

  ```bash
  cargo install cargo-make
  ```

  Note: cargo-make is not strictly necessary, just convenient. The underlying cargo commands can be inferred from the `Makefile.toml` file.

- Build targets `bryggio-supervisor`, `bryggio-cli` from source.

  ```bash
  git clone git@github.com:BryggansBryggeri/bryggio.git bryggio
  cd bryggio
  cargo make --no-workspace build
  ```

### Install NATS server

either from the [website](https://nats.io/download/) or by running (rather buggy right now)

```bash
cargo run --bin bryggio-cli install
```

Installation is a fancy word for simply downloading the `nats-server` executable.
Do this by manual download from the link above, or the `bryggio-cli install` command.
The latter figures out what OS you are running, and downloads the corresponding executable.
This is the preferred method, the end-goal is to use the CLI for a complete install of the entire BryggIO app.
It is though -- as stated -- still in development.

## Configuration

- **BryggIO config:** JSON file which specifies general settings, and importantly **the path to the `nats-server` binary** (that you downloaded in the install step).
  See `sample-bryggio.json` for an example.

Check out the sample config in this repo for usage.

## Run

The `bryggio-supervisor`, starts up a `nats-server` in a separate process and then runs a supervisor pub sub client which,
listening to special command subjects, starts and stops other clients like sensors, actors and controllers.

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
rustup target add armv7-unknown-linux-musleabihf
```

and add the target to `~/.cargo/config`

```
[target.armv7-unknown-linux-musleabihf]
linker = "arm-linux-gnueabihf-gcc-7"
```

Build the required executables

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
