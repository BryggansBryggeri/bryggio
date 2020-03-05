use bryggio_cli;
use bryggio_cli::opts::Opt;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    let api = bryggio_cli::Api::new(opt.ip, opt.port);
    match api.send(&opt.command) {
        Ok(response) => {
            if response.success {
                match response.result {
                    Some(result) => println!("Result: {}", result),
                    None => println!("Successful"),
                }
            } else {
                println!("Message: {}", response.message.unwrap());
            }
        }
        Err(err) => {
            println!("Error sending command: {}", err);
        }
    };
}
