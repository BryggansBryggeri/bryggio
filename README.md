# BRYGGIO

BRYGGANS BRYGGERI's very own brewery software.

Currently under heavy developement.
The goal is to develop a stand-alone backend to which any client can send http requests to and thereby control the brewery hardware.

## Development

 - Install rust from [here](https://www.rust-lang.org/tools/install).

 - Rocket needs nightly build so run
   ```bash
   rustup override set nightly
   ```

 - Clone the repo and run it
   ```bash
   git clone git@github.com:BryggansBryggeri/bryggio.git bryggio
   cd bryggio
   cargo run --bin bryggio-server
   ```

## Run on Rbpi

Build for rbpi (Needs arm-compatible rust toolchain installed)

```bash
cargo build --target=armv7-unknown-linux-gnueabihf
```

Move the resulting binary (`target/armv7-unknown-linux-gnueabihf/<build-mode>/bryggio-server`) to the rbpi.

Also needed: A `Rocket.toml` and a `Bryggio.toml` config file (Rocket dependency is being removed).

On the rbpi, currently the config files need to be in the same directory as the binary, then:

```bash
sudo ./bryggio-server
```
`sudo` is required for gpio manipulation.


## Control

Standard is to use a simple PID controller. This is slightly nihilistic given the almost complete model information one usually has when dealing with a simple brewery.
The arguments in favour of PID control are: ignorance, efficiency, laziness; what if we disregard some information? The system is slow and simple and it works fine.

`bryggio` takes the opposite approach: Given a simple and slow system, we have the possibility to have a complicated control. Furthermore, a slow system demands the best possible control in order to save precious time and energy.

Due to the inherent inertia in the objective (heating a lot of water), we can have quite long time between signal updates, providing time to calculate a nearly optimal control sequence.

### Literature
(http://www.iiisci.org/journal/CV$/sci/pdfs/ZA191KB18.pdf)

(http://kchbi.chtf.stuba.sk/upload_new/file/Miro/Proc%20problemy%20odovzdane%20zadania/%C5%A0uhaj/Predictive%20modelling%20of%20brewing%20fermentation%20from%20knowledge-based%20to%20black-box%20models.pdf)

(https://scholarworks.uni.edu/cgi/viewcontent.cgi?article=1661&context=etd)
