pub mod login;
pub mod logout;
pub mod register;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegistrationData {
    pub username: String,
    pub password: String,
    pub repeat_password: String,
    pub invitation_code: String,
    pub ip_address: String,
    pub mac_address: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegistrationResponse {
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginResponse {
    pub message: String,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginSuccessResponse {
    pub message: String,
    pub username: String,
    pub session_id: Uuid,
}

impl Validate for RegistrationData {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.username.len() < 3 || self.username.len() > 16 {
            let mut error = ValidationError::new("length");
            error.message = Some(
                "Username must be between 3 and 16 characters"
                    .to_string()
                    .into(),
            );
            errors.add("username", error);
        }
        if self.password.len() < 6 || self.password.len() > 16 {
            let mut error = ValidationError::new("length");
            error.message = Some(
                "Password must be between 6 and 16 characters"
                    .to_string()
                    .into(),
            );
            errors.add("password", error);
        }
        if self.repeat_password.len() < 6 || self.repeat_password.len() > 16 {
            let mut error = ValidationError::new("length");
            error.message = Some(
                "Repeat password must be between 6 and 16 characters"
                    .to_string()
                    .into(),
            );
            errors.add("repeat_password", error);
        }
        if self.invitation_code.len() < 3 || self.invitation_code.len() > 16 {
            let mut error = ValidationError::new("length");
            error.message = Some(
                "Invitation code must be between 3 and 16 characters"
                    .to_string()
                    .into(),
            );
            errors.add("invitation_code", error);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for LoginData {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.username.len() < 3 || self.username.len() > 16 {
            let mut error = ValidationError::new("length");
            error.message = Some(
                "Username must be between 3 and 16 characters"
                    .to_string()
                    .into(),
            );
            errors.add("username", error);
        }
        if self.password.len() < 6 || self.password.len() > 16 {
            let mut error = ValidationError::new("length");
            error.message = Some(
                "Password must be between 6 and 16 characters"
                    .to_string()
                    .into(),
            );
            errors.add("password", error);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn map_validation_errors(errors: ValidationErrors) -> String {
    let mut result = String::new();
    for (_, field_errors) in errors.field_errors() {
        for error in field_errors {
            result.push_str(&format!(
                "{} ",
                error.message.as_ref().unwrap_or(&Cow::from("Invalid value"))
            ));
        }
    }
    result.trim().to_string()
}
