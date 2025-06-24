use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{Request, request::{FromRequest, Outcome}};
use rocket::{get, post, launch, routes};

use std::env;


mod structs;
mod utils;
use crate::structs::{Response, SignUpRequest, SignInRequest, SignInResponse};
//use crate::utils;


#[get("/ping")]
fn ping() -> Json<Response<String>> {
    let b = env::current_dir().unwrap();
    let v = b.display();
    Json(Response::error(format!("Curdir {v}")))
}

#[post("/signup", format="json", data="<signup_d>")]
fn signup(signup_d: Json<SignUpRequest>) -> Json<Response<String>> {
    let signup_data = signup_d.0;
    if !signup_data.is_valid(){
        return Json(Response::error("validation error".to_string()));
    }
    if !utils::create_new_user(&signup_data){
        return Json(Response::error("saving error".to_string())); 
    }
    return Json(Response::ok("Signup successful".to_string()));
}

#[post("/signin", format="json", data="<signin_d>")]
fn signin(signin_d: Json<SignInRequest>) -> (Status, Json<Response<SignInResponse>>) {
    let signin_data = signin_d.0;
    if !utils::check_user(&signin_data){
        return (Status::NotFound, Json(Response::error("wrong credentials".to_string()))); 
    }

    let jwt = utils::get_jwt(&signin_data);

    return (Status::Ok, Json(Response::ok(SignInResponse{token:jwt})));
}


struct AuthorizationHeader<'a>(&'a str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizationHeader<'r>{
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            Some(ah) => Outcome::Success(AuthorizationHeader(ah)),
            None => Outcome::Success(AuthorizationHeader("")),
        }
    }
} 

#[post("/crashes/new", format="json", data="<crash_d>")] //get token from authoriation header
fn new_crash<'r>(crash_d: Json<structs::CrashReport>, auth_header: AuthorizationHeader<'_>) -> Json<Response<String>> {
    let mut token = auth_header.0;
    token = token.strip_prefix("Bearer ").unwrap_or(token);
    if token.is_empty(){
        return Json(Response::error("No token provided".to_string()));
    }
    
    let user = match utils::check_jwt(&token.to_string()){
        Some(claims) => {
            // Here you would typically save the crash report to a database or log it
            println!("Received crash report from user: {}", claims.sub);
            claims.sub
        },
        None => {
            return Json(Response::error("Invalid token".to_string()));
        }
    };
    if utils::save_crash_report(&user, &crash_d.0){
        Json(Response::ok("Crash report received".to_string()))
    }
    else{
        Json(Response::error("Crash report not inserted".to_string()))
    }
}

#[get("/crashes/all", format="json")]
fn get_crashes<'r>(auth_header: AuthorizationHeader<'_>) -> Json<Response<Vec<structs::CrashReport>>> {
    let mut token = auth_header.0;
    token = token.strip_prefix("Bearer ").unwrap_or(token);
    if token.is_empty(){
        return Json(Response::error("No token provided".to_string()));
    }
    
    let user = match utils::check_jwt(&token.to_string()){
        Some(claims) => claims.sub,
        None => {
            return Json(Response::error("Invalid token".to_string()));
        }
    };
    
    let crashes = utils::retrieve_all_crashed(&user);
    Json(Response::ok(crashes))
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/",
        routes![ping, signup, signin, new_crash, get_crashes]
    )
}