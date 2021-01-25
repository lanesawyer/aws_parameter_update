//! # AWS Parameter Update Library
//!
//! `aws_parameter_update` is a small tool used to quickly update simple AWS Parameters
#![feature(try_trait)]

#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]

mod parameter;

use log::{error, info};
pub use parameter::Parameter;
use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use std::fs::File;
use std::io::prelude::Read;
use std::{error::Error, iter::Map};
use yaml_rust::YamlLoader;

/// Updates AWS Parameters from a YAML file
///
/// # File Structure
/// The file structure for updating paramters is as follows:
/// ```yaml
/// - name: "new_parameter"
///   value: "Example parameter"
///   description: "An example of an unsecure parameter"
///   is_secure: false
/// - name: "new_secure_parameter"
///   value: "$uper$ecretP@$$W0rd"
///   description: "An example of a secure parameter"
///   is_secure: true
/// ```
///
/// # Example
///
/// ```should_panic because there is no file
/// let filename = "non_existing_file.yaml";
///
/// match tokio_test::block_on(aws_parameter_update::update_from_file(filename)) {
///     Ok(_) => {
///         println!("Parameter update from file {} finished", filename);
///     }
///     Err(error) => {
///         println!("Parameter updated from file {} failed: {}", filename, error);
///     }
/// };
/// ```
pub async fn update_from_file(filename: &str) -> Result<(), Box<dyn (Error)>> {
    let parameters_from_yaml = read_parameters_yaml(&filename)?;

    update_parameters(parameters_from_yaml).await
}

/// Updates AWS Parameter from calling function input
///
/// # Example
///
/// ```
/// let name = "name".to_string();
/// let value = "value".to_string();
/// let description = "description".to_string();
/// let is_secure = true;
///
/// match tokio_test::block_on(aws_parameter_update::update_parameter(&name, &value, &description, is_secure)) {
///     Ok(_) => {
///         println!("Parameter update finished");
///     }
///     Err(error) => {
///         println!("Parameter update failed: {}", error);
///     }
/// };
/// ```
pub async fn update_parameter(
    name: &str,
    value: &str,
    description: &str,
    is_secure: bool,
) -> Result<(), Box<dyn (Error)>> {
    update_parameters(vec![Parameter::new(name, value, description, is_secure)]).await
}

/// Updates AWS Parameters from calling function input
///
/// # Example
///
/// ```
/// use aws_parameter_update::Parameter;
///
/// let parameters_to_update = vec![Parameter::new(
///         "firstName",
///         "firstValue",
///         "firstDescription",
///         true,
///     ),
///     Parameter::new(
///         "secondName",
///         "secondValue",
///         "secondDescription",
///         false,
///     )];
///
/// match tokio_test::block_on(aws_parameter_update::update_parameters(parameters_to_update)) {
///     Ok(_) => {
///         println!("Parameter updates finished");
///     }
///     Err(error) => {
///         println!("Parameter updates failed: {}", error);
///     }
/// };
/// ```
pub async fn update_parameters(parameters: Vec<Parameter>) -> Result<(), Box<dyn (Error)>> {
    let client = SsmClient::new(Region::UsWest2);

    for parameter in parameters {
        match parameter.update(&client).await {
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

    // YamlLoader returns a "doc" which can have multiple YAML files in it,
    // hence the two iterators and the flattening
    let parameters = YamlLoader::load_from_str(&contents)?
        .into_iter()
        .map(|yaml_document| -> Map<_, _> {
            yaml_document.into_iter().map(|param| -> Parameter {
                Parameter::new(
                    param["name"].as_str().expect("name missing"),
                    param["value"].as_str().expect("value missing"),
                    param["description"].as_str().expect("description missing"),
                    param["is_secure"].as_bool().expect("is_secure missing"),
                )
            })
        })
        .flatten()
        .collect::<Vec<_>>();

    info!("Parameters YAML loaded");
    Ok(parameters)
}

mod tests {
    #[tokio::test]
    #[should_panic]
    async fn test_update_from_file() {
        let result = crate::update_from_file("missing_file.yaml").await;

        assert_ne!(result.unwrap(), ());
    }

    #[tokio::test]
    async fn test_update_parameter() {
        let result = crate::update_parameter("name", "value", "description", true).await;

        assert_eq!(result.unwrap(), ());
    }

    #[tokio::test]
    async fn test_update_parameters() {
        let result = crate::update_parameters(vec![crate::Parameter::new(
            "name",
            "value",
            "description",
            true,
        )])
        .await;

        assert_eq!(result.unwrap(), ());
    }
}
