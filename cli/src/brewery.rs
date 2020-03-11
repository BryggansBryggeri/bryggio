use crate::opts::BreweryOpt;
use crate::send;
use log::{debug, error, info};
pub fn process_command(command: &BreweryOpt) {
    match send::<f32>(&command.url().unwrap()) {
        Ok(response) => {
            if response.success {
                match response.result {
                    Some(result) => info!("Result: {}", result),
                    None => info!("Successful"),
                }
            } else {
                debug!("Message: {}", response.message.unwrap());
            }
        }
        Err(err) => {
            error!("Error sending command: {}", err);
        }
    };
}
