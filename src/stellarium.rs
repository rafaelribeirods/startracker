use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StellariumError {
    #[error("Could not get information about the selected object. Check if Stellarium's Remote Control is running on port {port}")]
    RequestError { port: u16 },
    #[error("Uh oh! Something unexpected happened.")]
    UnexpectedError,
    #[error("Select an object on Stellarium!")]
    ObjectNotFoundError,
    #[error("Could not parse response from Stellarium's Remote Control API")]
    UnableToParseError,
    #[error("The selected object cannot be tracked because it is not above the horizon. Please, select another object")]
    ObjectNotAboveHorizon,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StellariumObject {
    #[serde(alias = "above-horizon")]
    pub above_horizon: bool,
    #[serde(alias = "localized-name")]
    pub localized_name: String,
    pub name: String,
    #[serde(alias = "object-type")]
    pub object_type: String,
    pub altitude: f64,
    pub azimuth: f64,
}

pub async fn get_object(port: u16) -> Result<StellariumObject, StellariumError> {
    match reqwest::get(format!("http://localhost:{}/api/objects/info?format=json", port)).await {
        Ok(response) => match response.status() {
            reqwest::StatusCode::OK => match response.json::<StellariumObject>().await {
                Ok(selected_object) => {
                    if !selected_object.above_horizon {
                        return Err(StellariumError::ObjectNotAboveHorizon)
                    }
                    Ok(selected_object)
                },
                Err(_) => Err(StellariumError::UnableToParseError),
            },
            reqwest::StatusCode::NOT_FOUND => Err(StellariumError::ObjectNotFoundError),
            _ => Err(StellariumError::UnexpectedError),
        },
        Err(_) => Err(StellariumError::RequestError{ port }),
    }
}