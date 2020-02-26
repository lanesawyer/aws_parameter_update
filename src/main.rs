#![feature(try_trait)]

mod parameter;

#[cfg(test)]
mod tests;

extern crate clap;
extern crate rusoto_core;
extern crate rusoto_ssm;
extern crate yaml_rust;

use parameter::Parameter;

use clap::{App, Arg};
use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use std::error::Error;
use std::fs::File;
use std::io::prelude::Read;
use yaml_rust::{Yaml, YamlLoader};

fn main() {
    println!("Starting AWS parameter updates");

    let input_filename = get_filename_from_args();
    println!("Using input file: {}", input_filename);

    let parmeters_from_yaml = read_parameters_yaml(input_filename).unwrap();
    println!("Parameters YAML loaded");

    update_parameters(parmeters_from_yaml);
    println!("Parameters update finished running");
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
    filename.to_string()
}

fn read_parameters_yaml(filename: String) -> Result<Vec<Yaml>, Box<dyn (Error)>> {
    let mut file = File::open(filename).expect("Unable to open parameter input file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read parameter input file");

    let docs = YamlLoader::load_from_str(&contents)?;

    let parameters = docs[0].as_vec().unwrap().to_vec();
    Ok(parameters)
}

fn update_parameters(parameters_from_yaml: Vec<Yaml>) {
    let client = SsmClient::new(Region::UsWest2);

    for parameter in parameters_from_yaml {
        match Parameter::update(parameter, &client) {
            Ok(parameter_name) => println!("Parameter {} processed", parameter_name),
            Err(error) => println!("Parameter not updated: {:?}", error),
        }
    }
}
