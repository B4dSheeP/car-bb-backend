//structs.rs

use rocket::serde::{Serialize, Deserialize, json};

use crate::utils;

#[derive(Serialize)]
pub struct Response<T>{
    pub status: String,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUpRequest{
    pub username: String,
    pub password: String, 
    pub password2: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignInRequest{
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignInResponse{
    pub token: String,
}

impl SignUpRequest{

    pub fn is_valid(&self) -> bool{
        return utils::is_email(&self.username) && 
            utils::is_password_ok(&self.password) && 
                self.password == self.password2;
    }

}

impl<T> Response<T>{
    pub fn error(message: String) -> Self{
        Self{
            status: "error".to_string(),
            message: Some(message), 
            data: None
        }
        
    }

    pub fn ok(data: T) -> Self{

        Self{
            status: "ok".to_string(),
            message: None,
            data: Some(data)
        }
        
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoggedUserClaims{
    pub sub: String, 
    pub exp: u64,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct CrashReport{
    pub timestamp: u64,
    pub accel_data: Vec<AccelData>,
    pub gps_data: Vec<GpsData>,
}

impl CrashReport {
    pub fn get_serialized_json(&self) -> String {
        json::to_string(self).unwrap_or("Error serializing crash report".to_string())
    }

    pub fn get_deserialized_json(json_str: &str) -> Option<Self> {
        json::from_str(json_str).ok()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccelData {
    instant: u64,
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GpsData {
    instant: u64,
    latitude: f32,
    longitude: f32,
    altitude: f32,
    speed: f32,
}