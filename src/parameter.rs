use log::{error, info};
use rusoto_core::RusotoError;
use rusoto_ssm::GetParameterRequest;
use rusoto_ssm::PutParameterRequest;
use rusoto_ssm::{Ssm, SsmClient};
use std::option::NoneError;

/// Parameter struct
///
/// # Example
/// ```
/// use aws_parameter_update::Parameter;
/// let parameter = Parameter {
///     name: "example_name".into(),
///     value: "example_value".into(),
///     description: "example_description".into(),
///     is_secure: false    
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Parameter {
    /// `name` corresponds to the AWS parameter name
    pub name: String,
    /// `value` is the parameter's value, stored as a String
    pub value: String,
    /// `description` is extra text used to clarify the use of a parameter
    pub description: String,
    /// 'is_secure' toggles whether the parameter should be encrypted
    pub is_secure: bool,
}

impl Parameter {
    /// Creates a new parameter with arguments
    ///
    /// # Example
    /// ```
    /// use aws_parameter_update::Parameter;
    /// let parameter = Parameter::new("test_name", "test_value", "test_description", true);
    /// ```
    pub fn new<S>(name: S, value: S, description: S, is_secure: bool) -> Parameter
    where
        S: Into<String>,
    {
        Parameter {
            name: name.into(),
            value: value.into(),
            description: description.into(),
            is_secure,
        }
    }

    /// Updates a parameter
    ///
    /// # Example
    ///
    /// ```
    /// use aws_parameter_update::Parameter;
    /// use rusoto_core::Region;
    /// use rusoto_ssm::SsmClient;
    ///
    /// let client = SsmClient::new(Region::UsWest2);
    ///
    /// let parameter = Parameter {
    ///     name: "name".into(),
    ///     value: "value".into(),
    ///     description: "description".into(),
    ///     is_secure: true
    /// };
    ///
    /// match tokio_test::block_on(parameter.update(&client)) {
    ///     Ok(parameter_name) => println!("Parameter {} processed", parameter_name),
    ///     Err(_error) => println!("Parameter not updated"),
    /// }
    /// ```
    pub async fn update(&self, client: &SsmClient) -> Result<String, NoneError> {
        if self.needs_updating(client).await? {
            info!("Parameter {} needs updating", self.name);

            let parameter_request = self.to_put_parameter_request();

            match client.put_parameter(parameter_request).await {
                Ok(_parameter_result) => info!("Parameter {} successfully updated", self.name),
                Err(error) => error!("Parameter {} failed to update: {}", self.name, error),
            }
        } else {
            info!("Parameter {} does not need updating", self.name);
        }

        Ok(self.name.clone())
    }

    async fn needs_updating(&self, client: &SsmClient) -> Result<bool, NoneError> {
        match client.get_parameter(self.to_get_parameter_request()).await {
            Ok(parameter_result) => {
                let existing_value = parameter_result.parameter?.value?;

                info!(
                    "Found parameter {} with existing value: {}",
                    self.name, existing_value
                );

                Ok(self.value != existing_value)
            }
            Err(error) => {
                match error {
                    RusotoError::Credentials(error) => error!(
                        "Could not retreive parameter {}: {:?}",
                        self.name, error.message
                    ),
                    _ => error!("Could not retreive parameter {}: {:?}", self.name, error),
                };

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
                Some("SecureString".into())
            } else {
                Some("String".into())
            },
            overwrite: Some(true), // always overwrite or this utility is useless
            allowed_pattern: None,
            key_id: None,
            policies: None,
            tags: None,
            tier: None,
            data_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parameter::Parameter;

    #[test]
    fn get_parameter_request_sets_with_decryption_true() {
        let secure_parameter: Parameter =
            Parameter::new("test_name", "test_value", "test_description", true);

        let request = secure_parameter.to_get_parameter_request();

        assert_eq!(request.with_decryption.unwrap(), true);
    }

    #[test]
    fn get_parameter_request_sets_name() {
        let secure_parameter: Parameter =
            Parameter::new("test_name", "test_value", "test_description", true);

        let request = secure_parameter.to_get_parameter_request();

        assert_eq!(request.name, "test_name".to_string());
    }

    #[test]
    fn put_parameter_request_sets_overwrite_true() {
        let secure_parameter: Parameter =
            Parameter::new("test_name", "test_value", "test_description", true);

        let request = secure_parameter.to_put_parameter_request();

        assert_eq!(request.overwrite.unwrap(), true);
    }

    #[test]
    fn put_parameter_request_is_secure_sets_type_secure_string() {
        let secure_parameter: Parameter =
            Parameter::new("test_name", "test_value", "test_description", true);

        let request = secure_parameter.to_put_parameter_request();

        assert_eq!(request.type_, Some("SecureString".into()));
    }

    #[test]
    fn put_parameter_request_is_not_secure_sets_type_string() {
        let secure_parameter: Parameter =
            Parameter::new("test_name", "test_value", "test_description", false);

        let request = secure_parameter.to_put_parameter_request();

        assert_eq!(request.type_, Some("String".into()));
    }
}
