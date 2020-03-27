#![feature(try_trait)]

mod parameter;

use clap::ArgMatches;
use log::{error, info};
use parameter::Parameter;
use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use std::error::Error;
use std::fs::File;
use std::io::prelude::Read;
use yaml_rust::YamlLoader;

pub fn update_parameters<'a>(matches: ArgMatches<'a>) -> Result<(), Box<dyn (Error)>> {
    let filename = get_filename_from_args(matches);

    let parameters_from_yaml = read_parameters_yaml(filename)?;

    let client = SsmClient::new(Region::UsWest2);

    for parameter in parameters_from_yaml {
        match parameter.update(&client) {
            Ok(parameter_name) => info!("Parameter {} processed", parameter_name),
            Err(_error) => error!("Parameter not updated"),
        }
    }

    info!("Parameter update finished running");
    Ok(())
}

fn get_filename_from_args<'a>(matches: ArgMatches<'a>) -> String {
    let filename = matches.value_of("filename").unwrap_or("parameters.yaml");
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
