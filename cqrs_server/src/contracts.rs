use std::marker::PhantomData;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Command {
    type ErrorCodes: DeserializeOwned + Serialize + std::fmt::Debug;

    fn name() -> &'static str;
}

pub trait Query {
    type Result: Serialize + DeserializeOwned;
    fn name() -> &'static str;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ValidationError<T>
where
    T: Command,
{
    pub property_name: String,
    pub error_message: String,
    pub error_code: T::ErrorCodes,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CommandResult<T>
where
    T: Command + Serialize,
{
    pub validation_errors: Vec<ValidationError<T>>,
}

pub struct QueryResult<T>(Response, PhantomData<T>)
where
    T: Query;

impl<T> QueryResult<T>
where
    T: Query,
{
    pub fn new(data: &T::Result) -> Self {
        Self(Json(data).into_response(), PhantomData)
    }
}

impl<T> ValidationError<T>
where
    T: Command,
{
    pub fn new<P: ToString, E: ToString>(
        property: P,
        message: E,
        error_code: T::ErrorCodes,
    ) -> Self {
        Self {
            property_name: property.to_string(),
            error_message: message.to_string(),
            error_code,
        }
    }

    pub fn code(code: T::ErrorCodes) -> Self {
        Self::new("", "", code)
    }
}

impl<T> CommandResult<T>
where
    T: Command + Serialize,
    T::ErrorCodes: DeserializeOwned + Serialize + std::fmt::Debug,
{
    pub fn success() -> Self {
        Self {
            validation_errors: vec![],
        }
    }

    pub fn fail(validation_errors: Vec<ValidationError<T>>) -> Self {
        Self { validation_errors }
    }

    pub fn single_error(validation_error: ValidationError<T>) -> Self {
        Self::fail(vec![validation_error])
    }

    pub fn single_error_code(code: T::ErrorCodes) -> Self {
        Self::single_error(ValidationError::code(code))
    }

    pub fn was_successful(&self) -> bool {
        self.validation_errors.is_empty()
    }
}

impl<T> IntoResponse for CommandResult<T>
where
    T: Command + Serialize,
{
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

impl<T> IntoResponse for QueryResult<T>
where
    T: Query,
{
    fn into_response(self) -> axum::response::Response {
        self.0
    }
}

impl<T> std::ops::Try for CommandResult<T>
where
    T: Command + Serialize,
{
    type Output = ();
    type Residual = CommandResult<T>;

    fn from_output(_: Self::Output) -> Self {
        CommandResult::success()
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        if self.validation_errors.is_empty() {
            std::ops::ControlFlow::Continue(())
        } else {
            std::ops::ControlFlow::Break(self)
        }
    }
}

impl<T> std::ops::FromResidual<CommandResult<T>> for CommandResult<T>
where
    T: Command + Serialize,
{
    fn from_residual(residual: CommandResult<T>) -> Self {
        residual
    }
}
