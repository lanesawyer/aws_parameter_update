#![feature(try_trait)]

mod parameter;

#[cfg(test)]
mod tests;

extern crate clap;
extern crate log;
extern crate rusoto_core;
extern crate rusoto_ssm;
extern crate simple_logger;
extern crate yaml_rust;

use clap::{App, Arg};
use log::{error, info};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    info!("Starting AWS parameter updates");

    let matches = App::new("AWS Parameter Updater")
        .version("1.0")
        .author("Lane Sawyer <github@lanesawyer.dev>")
        .about("Allows you to update AWS Parameters using a YAML file")
        .arg(
            Arg::with_name("filename")
                .short("f")
                .long("filename")
                .value_name("filename")
                .help("Sets the input file to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("config_file")
                .help("Sets a custom config file, which currently does nothing")
                .takes_value(true),
        )
        .get_matches();

    match aws_parameter_update::update_parameters(matches) {
        Ok(_) => {
            info!("Parameter update finished");
            Ok(())
        }
        Err(error) => {
            error!("Parameter updated failed: {}", error);
            Err(error)
        }
    }
}
