#![feature(try_trait)]

#[cfg(test)]
mod tests;

use clap::{crate_version, App, Arg};
use log::{error, info, warn};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    info!("Starting aws_parameter_update...");

    let matches = App::new("AWS Parameter Updater")
        .version(crate_version!())
        .author("Lane Sawyer <github@lanesawyer.dev>")
        .about("Allows you to update AWS Parameters using a YAML file or directly from the CLI")
        .arg(
            Arg::with_name("filename")
                .help("Sets the input file to use")
                .short("f")
                .long("filename")
                .value_name("filename")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("name")
                .help("New parameter name")
                .short("n")
                .long("name")
                .value_name("name")
                .conflicts_with("filename")
                .requires_all(&["value", "description"])
                .takes_value(true),
        )
        .arg(
            Arg::with_name("value")
                .help("New parameter value")
                .short("v")
                .long("value")
                .value_name("value")
                .requires_all(&["name", "description"])
                .conflicts_with("filename")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("description")
                .help("New parameter decription")
                .short("d")
                .long("description")
                .value_name("description")
                .requires_all(&["name", "value"])
                .conflicts_with("filename")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("secure")
                .help("Stores the parameter securely")
                .short("s")
                .long("secure")
                .requires_all(&["name", "value", "description"])
                .conflicts_with("filename"),
        )
        .arg(
            Arg::with_name("config")
                .help("Sets a custom config file, which currently does nothing")
                .short("c")
                .long("config")
                .value_name("config_file")
                .takes_value(true),
        )
        .get_matches();

    if matches.is_present("filename") {
        let filename = matches.value_of("filename").unwrap();
        match aws_parameter_update::update_from_file(filename) {
            Ok(_) => {
                info!("Parameter update finished");
            }
            Err(error) => {
                error!("Parameter updated failed: {}", error);
            }
        };
    } else if matches.is_present("name") {
        let name = matches.value_of("name").unwrap();
        let value = matches.value_of("value").unwrap();
        let description = matches.value_of("description").unwrap();
        let is_secure = matches.is_present("secure");

        match aws_parameter_update::update_parameter(name, value, description, is_secure) {
            Ok(_) => {
                info!("Parameter update finished");
            }
            Err(error) => {
                error!("Parameter updated failed: {}", error);
            }
        };
    } else {
        warn!("No input was provided. Use -h or --help to see valid input options")
    }

    info!("Ending AWS parameter updates");

    Ok(())
}
