# API DOCUMENTATION

## Structure

### Communication
The brewery computer (currently a raspberry pi) runs a binary program called
`bryggio-server` which controls the brewery hardware and exposes an API in the
form of a webserver listening on a single port.

The hardware control and webserver run in separate threads.

The hardware control thread runs a never returning main loop
which waits and listens for commands from the webserver
and processes them as they come in.
Some commands spawns additional threads,
e.g. for starting an indefinite control of a sensor, actor pair.

The commands are sent as http requests in the form of:

`<ip_address_of_device>:<port>/<command_name>?<param_key_1>=<param_val_1>&<param_key_2>=<param_val_2>`

### Routing

The http requests are routed in a very picky way.
Anything that cannot be properly matched to a *route*
(see `bryggio::routes::backend`)
will default to a 404 not found error.

That includes parsing of the query parameters into strict rust types, meaning that:

`GET "/set_target_signal?controller_id=dummy&new_target_signal=10"`

works, but

`GET "/set_target_signal?controller_id=dummy&new_target_signal=aba"`

will be routed to 404, since `aba` cannot be parsed to a float value.

### Processing

In each route, a variant of the enum `bryggio::brewery::Command` is created,
where each variant holds the necessary data for the command.
For commands that require some query parameters, they are parsed from the http request.

The command is then sent to the hardware thread and processed accordingly.

### Response

A response struct `bryggio::api::response` is created on the form:

```rust
pub struct Response {
    pub success: bool,
    pub result: Option<f32>,
    pub message: Option<String>,
}
```

which is serialised to a Json string and sent back to the caller.
The Json string will have keys corresponding to the properties in the struct.

Requests which are not routed and are caught by the 404 route will
result in just the same `Response` struct.

**Note:** This struct will likely change to look more like the brewery command enum,
but the only thing that will change for the frontend is the possible keys in the
Json string.

### Control process

The only slightly complicated part in the program architecure are the *control processes*.
A control process is created with a pair of actor and sensor and runs in a separate thread.
The thread first locks the actor so that no other object can access it
and then enters its own main loop where in every iteration the controller:

1. Locks the controller, preventing access to it.
1. Reads a measurement from the sensor.
1. Updates the control signal using the new measurement and the target signal.
1. Sends the new control signal to the actor.
1. Unlocks the controller and sleeps for a fixed number of milliseconds (1000).
During this window the state of the controller can be changed by sending commands.

When a command is sent to an active controller it waits until it sleeps and then communicates with it.
This means that in the worst case scenario, the response time will be as long as the sleep time.

## Current hardcoded status

Since the dynamics are only partially implemented, some registration of sensors and actors
are hardcoded into the binary with the following members:

### Sensors

| sensor_id | Sensor type |
| ----------- | ----------- |
| "dummy"     | `bryggio::sensor::dummy::Sensor`     |
| "cpu"       | `bryggio::sensor::cpu_temp::CpuTemp` |

### Actors

| actor_id | Actor type |
| ----------- | ----------- |
| "dummy"     | `bryggio::actor::dummy::Actor`       |

You can also add DSB1820 temp sensors and GPIO actors via the config file `Bryggio.toml`.
However, while it is fine to simply add them, they do not work unless you are on a properly configured rbpi.

## Commands

The API consists of the following commands.
Note that a command can be unimplemented in two ways:

1. The request is not routed, results in a `Response` struct:
```rust
Response {
    success: false,
    result: None,
    message: "Error 404: <request> is not a valid API call",
}
```
1. The request is routed but the actual functionality is not in place yet,
results in:
```rust
Response {
    success: false,
    result: None,
    message: "Not implemented yet",
}
```

The response only has a non-empty `message` if `success==false`.

### Start controller
Implemented

`GET "/start_controller?controller_id=<id>controller_type=<type>&sensor_id=<id>&actor_id<id>"`

Choose a pair of already registred sensor and actor and start controlling them.
There can only be one controller per actor at the same time,
Multiple controller can however use the same sensor.

The controller is created and sent to a new thread before the response is returned to the webserver.

Currently, the available types are listed in `control::ControllerType`.

Query parameters:

- `controller_id: String`
- `controller_type: String`
- `sensor_id: String`
- `actor_id: String`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | none    | none    |
| false   | none    | err     |

### Stop controller
Implemented

`GET "/stop_controller?controller_id=<id>"`

Stop an existing control process.
Waits until the controller sleeps and is unlocked.
Changes the state to inactive, which will cause the controller to exit its main loop,
return and join the thread into the main hardware thread.

Query parameters:

- `controller_id: String`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | none    | none    |
| false   | none    | err     |

### Set target signal
Implemented

`GET "/set_target_signal?controller_id=<id>&new_target_signal=<id>"`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | none    | none    |
| false   | none    | err     |

### Get measurement
Implemented

`GET "/get_measurement?sensor_id=<id>"`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | float32 | none    |
| false   | none    | err     |

### Get control signal
Implemented

`GET "/get_control_signal?control_id=<id>"`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | float32 | none    |
| false   | none    | err     |

### Get control target
Implemented

`GET "/get_control_target?control_id=<id>"`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | float32 | none    |
| false   | none    | err     |

### Add sensor
Not implemented

`GET "/add_sensor?sensor_id=<id>&sensor_type=<id>"`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | none    | none    |
| false   | none    | err     |

### List available sensors
Currently lists all DSB1820 sensors registered by the Rbpi

`GET "/list_avaialable_sensors"`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | [string]| none    |
| false   | none    | err     |

### Add actor
Not implemented

`GET "/add_sensor?sensor_id=<id>&sensor_type=<type>"`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | f32     | none    |
| false   | none    | err     |

### Get full state
Not implemented

The necessity of a full command is unclear.
Especially in an async setting.
Perhaps at startup though.

`GET "/get_full_state"`

Query parameters:

- `none`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | none    | ?       |
| false   | none    | err     |

### Get config

Returns the full contents of the configuration file:
`Bryggio.toml` in json format.

Never fails, but it follows the same Response structure as the fallible calls.

`GET "/get_brewery_name"`

Query parameters:

- `none`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | config::Config  | none    |
| (false  | none    | err)    |

### Get brewery name

Never fails, but it follows the same Response structure as the fallible calls.

`GET "/get_brewery_name"`

Query parameters:

- `none`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | string  | none    |
| (false  | none    | err)    |

### Get bryggio version

Never fails, but it follows the same Response structure as the fallible calls.

`GET "/get_bryggio_version"`

Query parameters:

- `none`

Response:

| success | result  | message |
| ------- | ------- | ------- |
| true    | string  | none    |
| (false  | none    | err)    |
