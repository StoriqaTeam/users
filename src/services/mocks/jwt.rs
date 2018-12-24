use failure::Error as FailureError;
use futures::IntoFuture;
use hyper::Headers;

use services::jwt::profile::{FacebookProfile, GoogleProfile};
use services::jwt::JWTProviderService;
use services::types::ServiceFuture;

#[derive(Debug, Clone, Copy)]
pub struct JWTProviderServiceMock;

impl JWTProviderService<GoogleProfile> for JWTProviderServiceMock {
    fn get_profile(&self, _url: String, _headers: Option<Headers>) -> ServiceFuture<serde_json::Value> {
        let profile = GoogleProfile {
            picture: "https://s3.eu-west-2.amazonaws.com/storiqa/img-tovPJk6pVcIC-large.png".to_string(),
            email: "user@mail.com".to_string(),
            name: "User".to_string(),
            given_name: "User".to_string(),
            family_name: Some("Userovsky".to_string()),
            verified_email: true,
        };
        Box::new(serde_json::to_value(profile).map_err(FailureError::from).into_future())
    }
}

impl JWTProviderService<FacebookProfile> for JWTProviderServiceMock {
    fn get_profile(&self, _url: String, _headers: Option<Headers>) -> ServiceFuture<serde_json::Value> {
        let profile = FacebookProfile {
            id: "user_id".to_string(),
            email: "user@mail.com".to_string(),
            gender: Some("Male".to_string()),
            first_name: "User".to_string(),
            last_name: Some("Userovsky".to_string()),
            name: "User".to_string(),
        };
        Box::new(serde_json::to_value(profile).map_err(FailureError::from).into_future())
    }
}
