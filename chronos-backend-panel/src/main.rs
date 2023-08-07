use actix_web::get;
use actix_web::middleware::Logger;
use actix_web::web::{scope, Query};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
// use api::token::token;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::{System, SystemExt};
use std::sync::{Arc, RwLock};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


static SYSTEM: OnceLock<Arc<RwLock<System>>> = OnceLock::new();

#[get("/auth")]
async fn auth() -> impl Responder {
    let endpoint = format!("https://discord.com/api/oauth2/authorize?client_id={DISCORD_CLIENT_ID}&redirect_uri={COMPUTED_REDIRECT_URI}&response_type=code&scope=identify%20email%20guilds");
    let mut response = HttpResponse::new(StatusCode::FOUND);
    response.headers_mut().insert(
        HeaderName::from_str("Location").unwrap(),
        HeaderValue::from_str(&endpoint).unwrap(),
    );
    response
}

#[derive(Deserialize)]
struct OAuthCode {
    code: String,
}

#[derive(Serialize)]
struct DiscordData {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub redirect_uri: String,
    pub code: String,
    pub scope: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    expires_in: usize,
    access_token: String,
    refresh_token: Option<String>,
}

#[derive(Serialize)]
struct CookieContent {
    disco_access_token: String,
    disco_refresh_token: String,
}

#[get("/callback")]
async fn auth_callback(code: Query<OAuthCode>) -> impl Responder {
    let code = &code.code;
    println!("{code}");
    let data = DiscordData {
        client_id: DISCORD_CLIENT_ID.to_string(),
        client_secret: DISCORD_CLIENT_SECRET.to_string(),
        grant_type: String::from("authorization_code"),
        redirect_uri: "http://localhost:5173".to_string(),
        code: code.to_string(),
        scope: String::from("identity email guilds"),
    };
    let discord_request = reqwest::Client::new();
    let request = discord_request
        .post("https://discord.com/api/oauth2/token")
        .body(format!("client_id={}&client_secret={}&grant_type=client_credentials&code={}&redirect_uri={}&scope=identify%20email%20guilds", data.client_id, data.client_secret, code, data.redirect_uri))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .unwrap();
    let text = request.text().await.unwrap();
    println!("{text}");
    let body: TokenResponse = serde_json::from_str(&text).unwrap();
    let request: TokenResponse = body;
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    // 10 minutes
    let access_token_expires_in = since_the_epoch.as_millis() + request.expires_in as u128;
    // 30 days
    // let refresh_token_expires_in = since_the_epoch.as_millis() + 30 * 24 * 60 * 60 * 1000;
    let mut response = HttpResponse::new(StatusCode::FOUND);
    response.headers_mut()
        .insert(HeaderName::from_str("set-cookie").unwrap(),
                HeaderValue::from_str(
                    &format!("disco_access_token={access_token}; Path=/; SameSite=Strict; Expires={access_token_expires_in}", access_token = request.access_token)
                    ).unwrap());
    response.headers_mut().insert(
        HeaderName::from_str("Location").unwrap(),
        HeaderValue::from_str("/").unwrap(),
    );
    response
}

#[get("/refresh")]
async fn auth_refresh(code: Query<OAuthCode>) -> impl Responder {
    let code = &code.code;
    println!("{code}");
    let data = DiscordData {
        client_id: DISCORD_CLIENT_ID.to_string(),
        client_secret: DISCORD_CLIENT_ID.to_string(),
        grant_type: String::from("refresh_token"),
        redirect_uri: format!("https://discord.com/api/oauth2/token?client_id={DISCORD_CLIENT_ID}&redirect_uri={COMPUTED_REDIRECT_URI}&response_type=code&scope=identify%20email%20guilds"),
        code: code.to_string(),
        scope: String::from("identity email guilds"),
    };
    let discord_request = reqwest::Client::new();
    let request = discord_request
        .post("https://discord.com/api/oauth2/token")
        .body(format!("client_id=\"{}\"&client_secret={}&grant_type=client_credentials&code={}&redirect_uri={}&scope=identify%20email%20guilds", data.client_id, data.client_secret, code, data.redirect_uri))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .unwrap();
    let request: TokenResponse = request.json().await.unwrap();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    // 10 minutes
    let access_token_expires_in = since_the_epoch.as_millis() + request.expires_in as u128;
    // 30 days
    // let refresh_token_expires_in = since_the_epoch.as_millis() + 30 * 24 * 60 * 60 * 1000;
    let mut response = HttpResponse::new(StatusCode::OK);
    response.headers_mut()
        .insert(HeaderName::from_str("set-cookie").unwrap(),
                HeaderValue::from_str(
                    &format!("disco_access_token={access_token}; Path=/; HttpOnly; SameSite=Strict; Expires={access_token_expires_in}", access_token = request.access_token)
                    ).unwrap());
    response.headers_mut().insert(
        HeaderName::from_str("Location").unwrap(),
        HeaderValue::from_str("/").unwrap(),
    );
    response
}

#[get("/signout")]
async fn auth_signout() -> impl Responder {
    let mut response = HttpResponse::new(StatusCode::FOUND);
    response.headers_mut()
        .insert(HeaderName::from_str("set-cookie").unwrap(),
                HeaderValue::from_str(
                    "[\"disco_access_token=deleted; Path=/; Max-Age=-1\", \"disco_refresh_token=deleted; Path=/; Max-Age=-1\"]"
                    ).unwrap());
    response.headers_mut().insert(
        HeaderName::from_str("Location").unwrap(),
        HeaderValue::from_str("/").unwrap(),
    );
    response
}

#[derive(Serialize)]
struct CPUCore {
    frequency: u64,
    cpu_usage: f32,
}

#[derive(Serialize)]
struct Disk {
    name: String,
    kind: String,
    mount_point: String,
    used_space: u64,
    total_space: u64,
}

#[derive(Serialize)]
struct ServerInformation {
    host_name: String,
    used_memory: u64,
    total_memory: u64,
    used_swap: u64,
    total_swap: u64,
    cores: Vec<CPUCore>,
    update: u64,
    load_average_one: f64,
    load_average_five: f64,
    load_average_fifteen: f64,
}

impl ServerInformation {
    fn get_info() -> Self {
        // let system = SYSTEM.get()
        // Self { host_name: (), used_memory: (), total_memory: (), used_swap: (), total_swap: (), cores: (), update: (), load_average_one: (), load_average_five: (), load_average_fifteen: () }
        unimplemented!();
    }
}

// #[get("/server")]
// async fn server_information() -> impl Responder {
//
// }

#[derive(Serialize)]
struct Session {
    name: String,
}

// #[get("/sessions")]
// async fn get_sessions() -> impl Responder {
//
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::new(".").load();
    env_logger::init();
    SYSTEM.set(Arc::new(RwLock::new(System::new_all()))).unwrap();
    HttpServer::new(|| {
        // let oauth = claims::AuthConfig::default();
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(scope("api")
            .service(auth)
            .service(auth_callback)
            .service(auth_refresh)
            .service(auth_signout))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}
