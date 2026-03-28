# Specification of software for the new brewery

## System overview
The brewery consists of a single _vessel_, called Vessel.
It has a basket insert so that it can act as both a mash tun and a boil kettle.

The vessel is connected with pipes, hoses and valves, via a pump, conducting fluid to allow different forms of circulation.

The vessel has a heater and (multiple) _sensors_, mainly temperature sensors.
The heater is essentially a binary _actor_ ON/OFF but pulse width modulation allows emulation of continuous power output on [0, 1].
The temperature sensor are of the types:
- PT100
- DS18B20
They continuously (and in principle asynchronously) provide temperature measurements at different parts of the vessels or pipes

There is a cooling system, by which the pump drives hot wort through a heat exchanger.
Future extensions are likely to be of:
- Automatic valves
- Automatic pump control
- Additional sensors

Sensors and actors are connected to a Raspberry Pi.

## The brewing process
A typical brew follows these steps:

### Prep
Cold water is added to the vessel, acting as a mash tun, it is heated to ~70C.
We add malt and stir manually.

### Mashing
After the malt has settled, we start internal circulation in the mash tun.
The pump gently moves the fluid ("mash") from an outlet in the bottom to a spray-like outlet at the top.
The mash tun's heater is controlled so that it keeps the temperature of the mash very stable at some pre-defined target.

### Lautering
After about an hour of mashing, the temp of the mash is increased quite rapidly, then the internal basket is raised such that only liquid remains.
Additional hot water is manually poured over the basket to draw out all the sugar contents.

### Boiling.
When there is enough liquid to safely run the heater again, the vessel starts acting as a boil kettle. The fluid (now called "wort") is heated to boiling point.
The wort is boiled for about an hour, adding ingredients such as hops as the particular recipe calls for.

### Cooling and pitching
The wort is cooled by pumping it through a heat exchanger.
When sufficiently cooled, we do a final transfer to a fermentation tank and the brewing process is done.

## Architecture
The backend software, called `bryggio`, that does the controlling is written in rust.
The codebase is a Cargo workspace consisting of multiple crates, the most prominent being:
- `bryggio_core`: Lib, containing logic and type defs. This is pure sync code, using functional constructs as much as possible;
- `bryggio_server`: the executable which actually runs on the device it is driven by a tokio main loop which polls sensors and command inputs and requests a new state from `bryggio_core`.

### Specific modelling
The design principle is a tight coupling between the physical layout and the rust model. I.e., this is not a generic brewing software, but tailored to our specific hardware setup.
Trait objects, for instance describing a generic Sensor type, are avoided; instead enums represent all valid choices.

This informs the whole design. The physical brewery will is defined in rust code:
```rust
struct Brewery {
    vessel: Vessel
}

struct Vessel {
    top_temp_sensor: TempSensor 
    bottom_temp_sensor: TempSensor
    heater: Heater
}
```

### Control
The temperature of the liquid at the various stages of the process is controlled through software.
The control problem is simple, as in stable and slow.
We read a temperature and adapt the output of the heater to match the temperature target.
For fun, I want the possibility to add more complex control methods, such as model predictive control, but it is not strictly necessary.

The control processes are the most "dynamic" part of the system in that a controller (e.g. the heater controller) can select an arbitrary temperature sensor as measurement input.
This is still implemented with an enum over available temperature sensors.

Initially the controller will have two modes:
- PID: automatic control towards a set target;
- Manual: Simply set the heater power, independent of temperature readings.

Later we can experiment with other control methods.

### Hardware abstraction layer (HAL)
The tick loop is generic over a `Hal` trait (defined in `bryggio_core`), with two implementations in `bryggio_server`:
- **`RpiHal`** real hardware: reads DS18B20 via 1-Wire sysfs, PT100 via SPI/ADC, writes heaters via GPIO/PWM.
  Internally spawns background tasks to poll sensors and caches latest values behind a lock.
- **`MockHal`** software simulation: a thin wrapper against a physical model of the brewery.
  We strive to have realistic thermal dynamics (heating rate, passive cooling, sensor noise).
  The mock advances its physics model on each `tick` call using the last applied heater power.
  Supports injecting sensor failures.

The trait surface is minimal:
```rust
#[async_trait]
pub trait Hal: Send + Sync + 'static {
    async fn read_sensors(&self) -> SensorReadings;
    async fn apply_outputs(&self, outputs: &ActorOutputs) -> Result<(), HalError>;
}
```
Note: here we do have a trait, but it is used for polymorphism, not runtime dynamic dispatch of trait objects.
This enables full integration testing on a dev machine with no actual hardware support; the tick loop, PID controller, state broadcast, and error handling all run against the simulated environment.

### Async
The software will be designed as a thin async shell, with a synchronous core:
- Minimal `tokio` setup to handle sensors I/O, timers and communication;
- Control logic is still plain sync, this will go in a separete crate `bryggio_core`;
- The async layer calls in to the sync core on each tick. The ticks are driven by how fast the sensors can deliver new readings (if they are too fast we'll limit it to 10 Hz or so).

### Communication
We use `tokio::sync::watch/mpsc` channels for internal communication in the backend.

For external broadcast of the system's state to the UI we use Server-side events (SSE).
This is maximalist: the main loop gathers the full state and sends it in one piece.
To receive external signals (new target temp for instance) we use a simple http API. 
To achieve this we use the axum ecosystem to achieve:
- SSE
- HTTP API for commands
- Serving the UI files

### Error handling
We use `thiserror` to encode hierarchical and structured errors.
How to act on errors varies by case, but for instance, a dropped temp sensor should lead to actions like: kill the heat for that vessel.

### General design principles
- I want this code to be bullet-proof and never stop running, that means no panics, no unsafe:
  This should be enforced in every crate
  ```rust
  #![deny(unsafe_code)]
  #![warn(
      clippy::unwrap_used,
      clippy::expect_used,
      clippy::panic,
      clippy::indexing_slicing,
      clippy::as_conversions
  )] 
  ```
- New types when sensible, e.g. `struct Temperature(f32)` for additional type safety.
- Prefer functional style in `bryggio_core`: pure functions taking immutable data and returning new values (e.g. `tick(&BreweryState, ...) -> (BreweryState, ActorOutputs)`),
  iterator chains for data transformation, `fold` for command processing.
  Accept imperative style in `bryggio_server` where stateful I/O demands it (HAL, channels, the tick loop itself).

## UI
The frontend code is called `bryggui`

### SVG based
The UI will be based around a SVG representing an old timey Piping and instrumentation diagram P&ID
- Design SVG externally with well-named element IDs (`mash-tun-temp-incoming`, `boil-kettle-heater-power`, et c.)
- Embed it in the web page
- Use JS to bind live data to specific elements (valve states, temp sensors, et c).

### Control processes
As mentioned under `Architecture` I do want the flexibility to create a dynamic, say PID, controller from any source of sensor and actor, so there need at least be some dynamically growing `<div>`'s listing active control processes.
For this, we simply render a separate panel alongside the main SVG element.

### Charts and diagrams
Charts are provided by third party `chart.js`

### Framework
Rust -> Wasm is not worth it due to poor support for DOM manipulation and charting.
The best fit for this is apparently vanilla Svelte, so we go with that.

## Db
SQLite via `sqlx` (async). Single file database, no server process, trivial to back up (copy the file).
Initially, we only need a single table with the actual readings:
```sql
CREATE TABLE readings (
    brew_id     INTEGER NOT NULL REFERENCES brews(id), -- See meta table,
    timestamp   INTEGER NOT NULL,
    source      TEXT NOT NULL,           -- matches your enum: "mash_tun_temp_top", "boil_kettle_heater", etc.
    value       REAL NOT NULL,           -- temperature in °C, heater power as 0.0–1.0, etc.
    PRIMARY KEY (brew_id, timestamp, source)
);
```
but maybe also a meta table:
```sql
CREATE TABLE brews (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,           -- "Pisselager batch 3"
    recipe      TEXT,                    -- "Optional recipe file"
    started_at  INTEGER NOT NULL,           -- Timestamps are always in unix epoch
    ended_at    INTEGER,
    notes       TEXT
);
```
Buffer readings in memory and flush in a batched transaction every few seconds rather than writing every tick. This is kinder to the SD card and plays well with SQLite's transaction model. DB writes happen in the async layer, not the sync core.
We can also do some aggressive averaging, so that we only write values every 10s or so, even though the internal sensor sampling is much faster.
With current 1Hz sampling we write every value to the db.

## Logging
We set up proper logging from the start, use `tracing` crate.

## Recipe based state machine
Will be implemented at a later stage.
Build the manual system first, add recipe-driven automation later.

The brewing process always follows: Idle -> Prep -> Mashing -> Lautering -> Boiling -> Cooling -> Done.
A recipe defines parameters for each phase (target temps, durations, mash rest schedules), not the phases themselves.
The recipe state machine will be a source of target set points to the underlying control loops, same as the manual UI, just automated.

### Design constraints to preserve
To ensure the recipe system slots in cleanly later:
1. **The control loop takes setpoints, not instructions.** It receives a `(Heater, f64)` pair and doesn't care whether it came from the UI or a recipe.
2. **Phase is always tracked.** Even in manual mode, the brewer selects which phase they're in.
3. This structures DB readings and gives the future state machine a clean insertion point.
3. **A `Command` struct exists from day one.** The UI produces it manually now; the recipe system produces the same type later. No refactoring needed.

## Safety
Apparently, Claude is a little whiny safety Sally and thinks we should have watchdogs and shit. Maybe later.

## Deployment
The rust code is cross-compiled on a host machine using `cross` with a MUSL backend for statically linked binaries and copied to the Rbpi.
The UI code is similarly compiled to a minimal build artefact and shipped.

### CI / CD
None at all at this first stage.

## Sensor modelling
Sensors are modelled as separate enums per measurement kind, not as a shared hierarchy:
```rust
enum TempSensor {
    Pt100 { ... },
    Ds18b20 { address: String },
}

enum PressureSensor {
    SomeSpecificModel { ... },
}
```
Each enum variant encodes the hardware specifics.
Each produces its own newtype reading (`Temperature`, `Pressure`).
Type safety is enforced at the controller level: a `TempController` accepts `Temperature` and a `PressureController` accepts `Pressure`, you cannot accidentally cross-wire them.
Adding a new measurement kind means adding a new enum, a new newtype, a new controller wrapper, and a new field on `SensorReadings`. This is deliberate as it forces explicit integration.

## Physical modelling
For developing without hardware it is useful to have a somewhat accurate physical model of the brewery.
The principle is straightforward: Knowing the mass of the liquid and the power of the heater we can estimate how heating will affect the temperature of the liquid.
From measurements we can estimate more uncertain quantities such as heat loss.
The interesting part is modelling the distribution of heat within the vessel, which is done in two stages:
- Already implemented is a simpler two-zone model corresponding to temperature sensors at the top and bottom.
  It accounts for the power of the heater as well as the state of the pump (more or less mixing of the temperatures).
- Later, a more principled heat equation FEM model, this can be very sparse but I want a principled model as a learning project.

The more accurate the model the closer we can develop without hardware, in particular for developing complex control methods.
