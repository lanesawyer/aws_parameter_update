//! # AWS Parameter Update
//!
//! `aws_parameter_update` is a small tool used to quickly update simple AWS Parameters

#![feature(try_trait)]

mod parameter;

use log::{error, info};
use parameter::Parameter;
use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use std::error::Error;
use std::fs::File;
use std::io::prelude::Read;
use yaml_rust::YamlLoader;

/// Updates AWS Parameters from a YAML file
///
/// # File Structure
/// The file structure for updating paramters is as follows:
/// ```
/// - name: "new_parameter"
///   value: "Example parameter"
///   description: "An example of an unsecure parameter"
///   is_secure: false
/// - name: "new_secure_parameter"
///   value: "$uper$ecretP@$$W0rd"
///   description: "An example of an unsecure parameter
///   is_secure: true
/// ```
///
/// # Example
///
/// ```
/// match aws_parameter_update::update_from_file("parameters.yaml") {
///     Ok(_) => {
///         println!("Parameter update finished");
///     }
///     Err(error) => {
///         println!("Parameter updated failed: {}", error);
///     }
/// };
/// ```
pub fn update_from_file(filename: &str) -> Result<(), Box<dyn (Error)>> {
    let parameters_from_yaml = read_parameters_yaml(&filename)?;

    update_parameters(parameters_from_yaml)
}

/// Updates AWS Parameter from calling function input
///
/// # Example
///
/// ```
/// match aws_parameter_update::update_parameter("name", "value", "description", "is_secure") {
///     Ok(_) => {
///         println!("Parameter update finished");
///     }
///     Err(error) => {
///         println!("Parameter updated failed: {}", error);
///     }
/// };
/// ```
pub fn update_parameter(
    name: &str,
    value: &str,
    description: &str,
    is_secure: bool,
) -> Result<(), Box<dyn (Error)>> {
    update_parameters(vec![Parameter {
        name: name.to_string(),
        value: value.to_string(),
        description: description.to_string(),
        is_secure,
    }])
}

fn update_parameters(parameters: Vec<Parameter>) -> Result<(), Box<dyn (Error)>> {
    let client = SsmClient::new(Region::UsWest2);

    for parameter in parameters {
        match parameter.update(&client) {
            Ok(parameter_name) => info!("Parameter {} processed", parameter_name),
            Err(_error) => error!("Parameter not updated"),
        }
    }

    info!("Parameter update finished running");
    Ok(())
}

fn read_parameters_yaml(filename: &str) -> Result<Vec<Parameter>, Box<dyn (Error)>> {
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
