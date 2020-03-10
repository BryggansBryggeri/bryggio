use bryggio_cli;
use bryggio_cli::opts::Opt;
use structopt::StructOpt;

fn run_subcommand(opt: Opt) {
    match opt {
        Opt::Brewery(options) => {
            match bryggio_cli::send(&options.url().unwrap()) {
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
        Opt::Install => {}
    }
}

fn main() {
    let opt = Opt::from_args();
    run_subcommand(opt)
}
