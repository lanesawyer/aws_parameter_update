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

use parameter::Parameter;

use clap::{App, Arg};
use log::{error, info};
use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use std::error::Error;
use std::fs::File;
use std::io::prelude::Read;
use yaml_rust::YamlLoader;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    info!("Starting AWS parameter updates");

    let input_filename = get_filename_from_args();
    let parmeters_from_yaml = read_parameters_yaml(input_filename)?;
    update_parameters(parmeters_from_yaml);

    Ok(())
}

fn get_filename_from_args() -> String {
    let matches = App::new("AWS Parameter Updater")
        .version("1.0")
        .author("Lane Sawyer <github@lanesawyer.dev>")
        .about("Allows you to update AWS Parameters using a YAML file")
        .arg(
            Arg::with_name("input")
                .help("Sets the input file to use")
                .index(1),
        )
        .get_matches();

    let filename = matches.value_of("input").unwrap_or("parameters.yaml");
    info!("Using input file: {}", filename);
    filename.to_string()
}

fn read_parameters_yaml(filename: String) -> Result<Vec<Parameter>, Box<dyn (Error)>> {
    let mut file = File::open(filename).expect("Unable to open parameter input file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read parameter input file");

    let docs = YamlLoader::load_from_str(&contents)?;

    let parameters: Vec<Parameter> = docs[0]
        .as_vec()
        .unwrap()
        .to_vec()
        .iter()
        .map(|param| Parameter {
            name: param["name"].as_str().unwrap().to_string(),
            value: param["value"].as_str().unwrap().to_string(),
            description: param["description"].as_str().unwrap().to_string(),
            is_secure: param["is_secure"].as_bool().unwrap(),
        })
        .collect();

    info!("Parameters YAML loaded");
    Ok(parameters)
}

fn update_parameters(parameters_from_yaml: Vec<Parameter>) {
    let client = SsmClient::new(Region::UsWest2);

    for parameter in parameters_from_yaml {
        match parameter.update(&client) {
            Ok(parameter_name) => info!("Parameter {} processed", parameter_name),
            Err(error) => error!("Parameter not updated: {:?}", error),
        }
    }

    info!("Parameters update finished running");
}
