use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The name of the serial port on which the Arduino is connected
    pub serial_port: String,
    /// The port on which Stellarium's Remote Control API is running
    pub api_port: u16,
}

pub fn parse() -> Cli {
    return Cli::parse();
}