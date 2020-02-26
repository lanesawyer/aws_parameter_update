use rusoto_ssm::GetParameterRequest;
use rusoto_ssm::PutParameterRequest;
use rusoto_ssm::{Ssm, SsmClient};
use std::option::NoneError;
use yaml_rust::Yaml;

pub struct Parameter {
    name: String,
    value: String,
    description: String,
    is_secure: bool,
}

impl Parameter {
    pub fn update(param: Yaml, client: &SsmClient) -> Result<String, NoneError> {
        let parameter = Parameter {
            name: param["name"].as_str()?.to_string(),
            value: param["value"].as_str()?.to_string(),
            description: param["description"].as_str()?.to_string(),
            is_secure: param["is_secure"].as_bool()?,
        };

        if parameter.needs_updating(client)? {
            let parameter_request = parameter.to_put_parameter_request();

            match client.put_parameter(parameter_request).sync() {
                Ok(_parameter_result) => {
                    println!("Parameter {} successfully updated", parameter.name)
                }
                Err(error) => println!("Parameter {} failed to update: {}", parameter.name, error),
            }
        }

        Ok(parameter.name)
    }

    fn needs_updating(&self, client: &SsmClient) -> Result<bool, NoneError> {
        match client.get_parameter(self.to_get_parameter_request()).sync() {
            Ok(parameter_result) => {
                let existing_value = parameter_result.parameter?.value?;
                println!(
                    "Found parameter {} with existing value: {}",
                    self.name, existing_value
                );

                if self.value != existing_value {
                    println!("Parameter {} needs updating", self.name);
                    Ok(true)
                } else {
                    println!("Parameter {} does not need updating", self.name);
                    Ok(false)
                }
            }
            Err(error) => {
                println!("Could not retreive parameter {}: {:?}", self.name, error);
                panic!();
            }
        }
    }

    fn to_get_parameter_request(&self) -> GetParameterRequest {
        GetParameterRequest {
            name: self.name.clone(),
            with_decryption: Some(true), // always decrypt so we can comepare existing and new values
        }
    }

    fn to_put_parameter_request(&self) -> PutParameterRequest {
        PutParameterRequest {
            name: self.name.clone(),
            value: self.value.clone(),
            description: Some(self.description.clone()),
            type_: if self.is_secure {
                String::from("SecureString")
            } else {
                String::from("String")
            },
            overwrite: Some(true), // always overwrite or this utility is useless
            allowed_pattern: None,
            key_id: None,
            policies: None,
            tags: None,
            tier: None,
        }
    }
}
