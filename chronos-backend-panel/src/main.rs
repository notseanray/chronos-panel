use actix_web::get;
use serde_json::json;
use tokio::time::sleep_until;
use crate::authenticate_token::AuthenticationGuard;
use actix_web::middleware::Logger;
use actix_web::web::{scope, Query};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
mod authenticate_token;
mod models;
use std::ops::Add;
use std::process::Command;
use std::str::FromStr;
use std::sync::OnceLock;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
// use api::token::token;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::{System, SystemExt, CpuExt};
use std::sync::{Arc, RwLock};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

static DISCORD_CLIENT_ID: &str = "1137543889222389780";
static COMPUTED_REDIRECT_URI: &str = "http://localhost:8080/api/callback";
static DISCORD_CLIENT_SECRET: &str = "vKiBGSNaZiy2SH8RjjUx_WRtqdN-y0TL";

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
    uptime: u64,
    load_average_one: f64,
    load_average_five: f64,
    load_average_fifteen: f64,
}

impl ServerInformation {
    fn get_info() -> Option<Self> {
        let system = SYSTEM.get();
        if let Some(v) = system {
            let s = v.read().unwrap();
            let la = s.load_average();
            Some(Self {
                host_name: s.host_name().unwrap_or_default(),
                used_memory: s.used_memory(),
                total_memory: s.total_memory(),
                used_swap: s.used_swap(),
                total_swap: s.total_swap(),
                cores: s.cpus().iter().map(|x| CPUCore {
                    frequency: x.frequency(),
                    cpu_usage: x.cpu_usage(),
                }).collect(),
                uptime: s.uptime(),
                load_average_one: la.one,
                load_average_five: la.five,
                load_average_fifteen: la.fifteen
            })
        } else {
            None
        }
    }
}

// #[get("/server")]
// async fn server_information() -> impl Responder {
//
// }

#[derive(Serialize)]
struct Session {
    id: u32,
    name: String,
    creation: String,
    attached: bool,
}

impl Session {
    fn get_sessions() -> Result<Option<Vec<Self>>, Box<dyn Error>> {
        let data = String::from_utf8(Command::new("screen").arg("-ls").output()?.stdout)?;
        let mut sessions = Vec::new();
        for (i, line) in data.lines().enumerate() {
            if i == 0 || line.is_empty() {
                continue;
            }
            let session: Vec<&str> = line.split('.').collect();
            if let (Some(id), Some(rest)) = (session.first(), session.last()) {
                let rest: Vec<String> = rest.split('(').map(|x| x.replace(')', "").trim().to_string()).collect();
                if rest.len() != 3 {
                    continue;
                }
                sessions.push(Self {
                    id: id.parse::<u32>().unwrap_or_default(),
                    name: rest[0].to_string(),
                    creation: rest[1].to_string(),
                    attached: rest[3].to_lowercase() == "attached"
                });
            }
        }
        Ok(Some(sessions))
    }
}

#[get("/sessions")]
async fn get_sessions(_guard: AuthenticationGuard) -> impl Responder {
    json!(ServerInformation::get_info())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    SYSTEM.set(Arc::new(RwLock::new(System::new_all()))).unwrap();
    tokio::spawn(async move {
        loop {
            if let Some(v) = SYSTEM.get() {
                v.write().unwrap().refresh_all();
                // TODO replace with env var for time
                sleep_until(Instant::now().add(Duration::from_secs(5)).into()).await;
            }
        }
    });
    HttpServer::new(|| {
        // let oauth = claims::AuthConfig::default();
        let logger = Logger::default();
        let cors = Cors::permissive();
        App::new()
            .wrap(logger)
            .wrap(cors)
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
