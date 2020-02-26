use rusoto_ssm::GetParameterRequest;
use rusoto_ssm::PutParameterRequest;
use rusoto_ssm::{Ssm, SsmClient};
use std::option::NoneError;

pub struct Parameter {
    pub name: String,
    pub value: String,
    pub description: String,
    pub is_secure: bool,
}

impl Parameter {
    pub fn update(&self, client: &SsmClient) -> Result<String, NoneError> {
        if self.needs_updating(client)? {
            let parameter_request = self.to_put_parameter_request();

            match client.put_parameter(parameter_request).sync() {
                Ok(_parameter_result) => println!("Parameter {} successfully updated", self.name),
                Err(error) => println!("Parameter {} failed to update: {}", self.name, error),
            }
        }

        Ok(self.name.clone())
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
