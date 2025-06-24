
use rusqlite;
use regex;
use bcrypt;
use jsonwebtoken;

use crate::structs;

const DB_PATH: &str = "./database.db";
const SECRET: &str = "DEADBEAF";

pub fn create_new_user(signup_request: &structs::SignUpRequest) -> bool{
    if let Ok(connection) = rusqlite::Connection::open(DB_PATH){
        let query = "INSERT INTO users(email, password) VALUES(?, ?)";
        let password_hash = bcrypt::hash(&signup_request.password, 10).unwrap();
        //println!("password hashed {}", password_hash);
        let mut statement = connection.prepare(query).unwrap();
        if let Ok(_) = statement.execute(&[&signup_request.username,
            &password_hash]){
            return true;
        }
    }
    false
}

pub fn check_user(signin_request: &structs::SignInRequest) -> bool{
    if let Ok(connection) = rusqlite::Connection::open(DB_PATH){
        let query = "SELECT password FROM users WHERE email = ?";
        if let Ok(result_password) = connection.query_row(query, &[&signin_request.username], |r| r.get::<usize, String>(0)){
        //println!("found pass: {}", result_password);
            return match bcrypt::verify(&signin_request.password, result_password.as_str()){
                Ok(res) => res,
                _ => false
            }
        }
        else { return false; }
    }
    false
} 

pub fn get_jwt(signin_request: &structs::SignInRequest) -> String{
    let header = jsonwebtoken::Header::default();
    let claims = structs::LoggedUserClaims{
        sub: signin_request.username.clone(),
        exp: jsonwebtoken::get_current_timestamp() + 3600 * 24, // a day, for debugging
    };
    
    match jsonwebtoken::encode(&header, &claims, &jsonwebtoken::EncodingKey::from_secret(SECRET.as_ref())) {
        Ok(token) => token,
        Err(_) => "Error generating token".to_string(),
    }
}

pub fn check_jwt(token: &String) -> Option<structs::LoggedUserClaims> {
    match jsonwebtoken::decode::<structs::LoggedUserClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(SECRET.as_ref()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(decoded) => Some(decoded.claims),
        Err(_) => None,
    }
}

pub fn is_email(email: &String) -> bool {
    // A very basic email validation regex
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

pub fn is_password_ok(password: &String) -> bool{
    let mut res = true;
    let mut tmp = false;
    for c in password.chars(){
        if c.is_uppercase(){ tmp = true; }
    }
    res &= tmp;
    tmp = false;
    for c in password.chars(){
        if c.is_lowercase(){ tmp = true; }
    }
    res &= tmp;
    tmp = false;
    for c in password.chars(){
        if c.is_digit(10){ tmp = true; }
    }
    res &= tmp;
    tmp = false;
    for c in password.chars(){
        if "!@#$%^&*()_+-=[]{}|;:',.<>?/".contains(c){ tmp = true; }
    }
    res &= tmp;
    return res;
}


pub fn save_crash_report(user: &String, crash_report: &structs::CrashReport) -> bool {
    if let Ok(connection) = rusqlite::Connection::open(DB_PATH) {
        let timestamp_lb = (crash_report.timestamp / 100) * 100; //granularity deciseconds
        let timestamp_ub = timestamp_lb + 100;
        let query = "SELECT COUNT(*) FROM crashevents WHERE username = ? AND timestamp >= ? AND timestamp < ?";
        let count: u32 = connection.query_row(query, &[user, &(timestamp_lb.to_string()), &(timestamp_ub.to_string())], |row| row.get(0)).unwrap_or(0);
        if count > 0 {
            // A crash report for this user at this timestamp already exists
            return false;
        }
        else{
            let insert_query = "INSERT INTO crashevents(username, timestamp, data) VALUES(?, ?, ?)";
            let mut statement = connection.prepare(insert_query).unwrap();
            let json_data = crash_report.get_serialized_json();
            if let Ok(_) = statement.execute(&[user, &(crash_report.timestamp).to_string(), &json_data]) {
                return true;
            }
        }
    }
    false
}


pub fn retrieve_all_crashed(user: &String) -> Vec<structs::CrashReport> {
    let mut crash_reports = Vec::new();
    if let Ok(connection) = rusqlite::Connection::open(DB_PATH) {
        let query = "SELECT timestamp, data FROM crashevents WHERE username = ?";
        if let Ok(mut statement) = connection.prepare(query) {
            if let Ok(rows) = statement.query_map(&[user], |row| {
                let data: String = row.get(1).unwrap();
                let crash_report: structs::CrashReport = structs::CrashReport::get_deserialized_json(&data).unwrap_or_else(|| {
                    eprintln!("Error deserializing crash report data: {}", data);
                    structs::CrashReport {
                        timestamp: 0,
                        accel_data: Vec::new(),
                        gps_data: Vec::new(),
                    }
                });
                Ok(crash_report)
            }) {
                for report in rows {
                    crash_reports.push(report.unwrap());
                }
            }
        }
    }
    crash_reports
}