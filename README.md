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

- Implemented in rust, a strictly typed, compiled and safe language (no unsafe code is used in the core software),
  makes BryggIO very reliable. The compiler checks all edge cases and crashing the software is extremely difficult.
  As a bonus, it also makes BryggIO very fast and memory efficient.
- We aim to be very stable and use semantic versioning to communicate changes' impact.
  Note: We are currently in pre-release (<1.0) and during that period wild changes can occur.
- BryggIO is, and will remain, free and open source. We will keep backend and frontend separated so that any frontend can be used.

### Control

Standard is to use a simple PID controller. This is slightly nihilistic given the almost complete model information one usually has when dealing with a simple brewery.
The arguments in favour of PID control are: ignorance, efficiency, laziness; what if we disregard some information? The system is slow and simple and it works fine.

BryggIO takes the opposite approach: Given a simple and slow system, we have the possibility to have a complicated control. Furthermore, a slow system demands the best possible control in order to save precious time and energy.

Due to the inherent inertia in the objective (heating a lot of water), we can have quite long time between signal updates, providing time to calculate a nearly optimal control sequence.

## Development

 - Install rust from [here](https://www.rust-lang.org/tools/install).

 - Clone the repo and build the different targets
   ```bash
   git clone git@github.com:BryggansBryggeri/bryggio.git bryggio
   cd bryggio
   cargo make <target>
   ```
 - The different `cargo make` tasks are:
   TODO: List

## Run on Rbpi

Build for rbpi needs

- arm-compatible rust toolchain installed
- OpenSSL lib for arm

```bash
cargo make rbpi-build
```

Move the resulting binary (`target/armv7-unknown-linux-gnueabihf/<build-mode>/bryggio-supervisor`) to the rbpi.

Also needed: A `Bryggio.toml` config file.

On the rbpi, currently the config files need to be in the same directory as the binary, then:

```bash
sudo ./bryggio-supervisor
```
`sudo` is required for gpio manipulation.

### Literature
(http://www.iiisci.org/journal/CV$/sci/pdfs/ZA191KB18.pdf)

(http://kchbi.chtf.stuba.sk/upload_new/file/Miro/Proc%20problemy%20odovzdane%20zadania/%C5%A0uhaj/Predictive%20modelling%20of%20brewing%20fermentation%20from%20knowledge-based%20to%20black-box%20models.pdf)

(https://scholarworks.uni.edu/cgi/viewcontent.cgi?article=1661&context=etd)
