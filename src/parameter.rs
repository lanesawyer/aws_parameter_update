use log::{error, info};
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
            info!("Parameter {} needs updating", self.name);

            let parameter_request = self.to_put_parameter_request();

            match client.put_parameter(parameter_request).sync() {
                Ok(_parameter_result) => info!("Parameter {} successfully updated", self.name),
                Err(error) => error!("Parameter {} failed to update: {}", self.name, error),
            }
        } else {
            info!("Parameter {} does not need updating", self.name);
        }

        Ok(self.name.clone())
    }

    fn needs_updating(&self, client: &SsmClient) -> Result<bool, NoneError> {
        match client.get_parameter(self.to_get_parameter_request()).sync() {
            Ok(parameter_result) => {
                let existing_value = parameter_result.parameter?.value?;

                info!(
                    "Found parameter {} with existing value: {}",
                    self.name, existing_value
                );

                Ok(self.value != existing_value)
            }
            Err(error) => {
                error!("Could not retreive parameter {}: {:?}", self.name, error);
                Err(std::option::NoneError)
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

#[cfg(test)]
mod tests {
    use crate::parameter::Parameter;

    #[test]
    fn get_parameter_request_sets_with_decryption_true() {
        let secure_parameter: Parameter = Parameter {
            name: "test_name".to_string(),
            value: "test_value".to_string(),
            description: "test_description".to_string(),
            is_secure: true,
        };

        let request = secure_parameter.to_get_parameter_request();

        assert_eq!(request.with_decryption.unwrap(), true);
    }

    #[test]
    fn get_parameter_request_sets_name() {
        let secure_parameter: Parameter = Parameter {
            name: "test_name".to_string(),
            value: "test_value".to_string(),
            description: "test_description".to_string(),
            is_secure: true,
        };

        let request = secure_parameter.to_get_parameter_request();

        assert_eq!(request.name, "test_name".to_string());
    }

    #[test]
    fn put_parameter_request_sets_overwrite_true() {
        let secure_parameter: Parameter = Parameter {
            name: "test_name".to_string(),
            value: "test_value".to_string(),
            description: "test_description".to_string(),
            is_secure: true,
        };

        let request = secure_parameter.to_put_parameter_request();

        assert_eq!(request.overwrite.unwrap(), true);
    }

    #[test]
    fn put_parameter_request_is_secure_sets_type_secure_string() {
        let secure_parameter: Parameter = Parameter {
            name: "test_name".to_string(),
            value: "test_value".to_string(),
            description: "test_description".to_string(),
            is_secure: true,
        };

        let request = secure_parameter.to_put_parameter_request();

        assert_eq!(request.type_, "SecureString");
    }

    #[test]
    fn put_parameter_request_is_not_secure_sets_type_string() {
        let secure_parameter: Parameter = Parameter {
            name: "test_name".to_string(),
            value: "test_value".to_string(),
            description: "test_description".to_string(),
            is_secure: false,
        };

        let request = secure_parameter.to_put_parameter_request();

        assert_eq!(request.type_, "String");
    }
}