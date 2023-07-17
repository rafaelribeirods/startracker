use std::{process, time::Duration, thread::sleep, io::{self, Write}};

use cli::Cli;

mod cli;
mod stellarium;

#[tokio::main]
async fn main() {
    let mut duration: u64;
    let mut error: bool;
    let mut message: String;
    let mut last_object = String::new();

    let cli: Cli = cli::parse();

    match ctrlc::set_handler(move || {
        println!();
        println!("Stopping...");
        process::exit(0);
    }) {
        Ok(_) => {},
        Err(_) => { 
            println!();
            println!("Could not handle program exit...");
            process::exit(0);
        },
    };

    let mut serial_port = match serialport::new(cli.serial_port.clone(), 9600).timeout(Duration::from_secs(2)).open() {
        Ok(port) => port,
        Err(_) => {
            println!("Could not open serial port {}", cli.serial_port);
            process::exit(0);
        },
    };
    
    loop {
        duration = 2;
        error = false;
        match stellarium::get_object(cli.api_port).await {
            Ok(object) => {
                if object.name != last_object {
                    last_object = object.name.clone();
                    message = format!(
                        "Tracking {} ({}, {}): {}, {}",
                        object.localized_name,
                        object.name,
                        object.object_type,
                        format!("{:.3}", object.azimuth),
                        format!("{:.3}", object.altitude)
                    );
                    println!();
                    println!("{}", message);
                }
                
                if let Err(_) =  serial_port.write(format!("{},{}", format!("{:.3}", object.azimuth), format!("{:.3}", object.altitude)).as_bytes()) {
                    println!("Could not write data to serial port {}", cli.serial_port);
                    process::exit(0);
                }

                let mut serial_buf: Vec<u8> = vec![0; 7];
                if let Ok(_) = serial_port.read(serial_buf.as_mut_slice()) {
                    let contents = String::from_utf8_lossy(&serial_buf);
                    if contents == "msg_rec" {
                        print!(".");
                        io::stdout().flush().unwrap();
                    }
                }
            },
            Err(err) => {
                last_object = String::new();
                error = true;
                println!();
                println!("Error: {}", err);
                match err {
                    stellarium::StellariumError::RequestError { port: _ } => { process::exit(0) },
                    stellarium::StellariumError::UnexpectedError => { process::exit(0) },
                    stellarium::StellariumError::ObjectNotFoundError => { duration = 10 },
                    stellarium::StellariumError::UnableToParseError => { process::exit(0) },
                    stellarium::StellariumError::ObjectNotAboveHorizon => { duration = 10 },
                }
            },
        }
        
        if error {
            println!("Will try again in {} seconds.", duration);
        }

        sleep(Duration::from_secs(duration));
    }
}