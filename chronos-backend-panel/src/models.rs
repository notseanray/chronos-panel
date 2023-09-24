use std::{time::SystemTime, sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use std::env;
use actix_web::web;
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Config {
    pub client_origin: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_max_age: i64,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_redirect_url: String,
}

impl Config {
    pub fn init() -> Config {
        let client_origin = std::env::var("CLIENT_ORIGIN").expect("CLIENT_ORIGIN must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expires_in =
            std::env::var("TOKEN_EXPIRED_IN").expect("TOKEN_EXPIRED_IN must be set");
        let jwt_max_age = std::env::var("TOKEN_MAXAGE").expect("TOKEN_MAXAGE must be set");
        let oauth_client_id =
            std::env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID must be set");
        let oauth_client_secret = std::env::var("OAUTH_CLIENT_SECRET")
            .expect("OAUTH_CLIENT_SECRET must be set");
        let oauth_redirect_url = std::env::var("OAUTH_REDIRECT_URL")
            .expect("OAUTH_REDIRECT_URL must be set");

        Config {
            client_origin,
            jwt_secret,
            jwt_expires_in,
            jwt_max_age: jwt_max_age.parse::<i64>().unwrap(),
            oauth_client_id,
            oauth_client_secret,
            oauth_redirect_url,
        }
    }
}

pub struct AppState {
    pub init_ts: SystemTime,
    pub pool: Arc<Mutex<SqlitePool>>,
    verified_cache: VerifiedCache,
    pub env: Config,
}


#[derive(Default)]
struct VerifiedCache {
    pub token_cache: Arc<Mutex<HashMap<String, SystemTime>>>,
}

impl VerifiedCache {
    pub async fn insert(&self, token: String) {
        self.token_cache.lock().await.insert(token, SystemTime::now());
    }
    pub async fn check(&self, token: String) -> bool {
        self.token_cache.lock().await.entry(key)
    }
}

impl AppState {
    pub async fn init() -> AppState {
        AppState {
            init_ts: SystemTime::now(),
            pool: Arc::new(Mutex::new(
                SqlitePool::connect(&env::var("DATABASE_URL").expect("missing DATABASE_URL"))
                    .await
                    .unwrap(),
            )),
            verified_cache: VerifiedCache::default(),
            env: Config::init(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

/*
 *
 *
 * {
  "id": "80351110224678912",
  "username": "Nelly",
  "discriminator": "1337",
  "avatar": "8342729096ea3675442027381ff50dfe",
  "verified": true,
  "email": "nelly@discord.com",
  "flags": 64,
  "banner": "06c16474723fe537c283b8efa61a30c8",
  "accent_color": 16711680,
  "premium_type": 1,
  "public_flags": 64
}
 */

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub verified: i64,
    pub email: Option<String>,
    pub accent_color: Option<i64>,
}

impl User {
    pub async fn create(&self, data: &web::Data<AppState>) -> Result<(), ()> {
        let mut db = data.pool.lock().await.acquire().await.unwrap();
        if sqlx::query_as!(User,
            r#"
    INSERT INTO users (id, username, discriminator, avatar, verified, email, accent_color)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            self.id,
            self.username,
            self.discriminator,
            self.avatar,
            self.verified,
            self.email,
            self.accent_color,
        ).execute(&mut db).await.is_ok() {
            Ok(())
        } else {
            Err(())
        }
    }
    pub async fn get_by_id(&self, data: &web::Data<AppState>, id: String) -> Result<User, ()> {
         let mut db = data.pool.lock().await.acquire().await.unwrap();
            if let Ok(Some(v)) = sqlx::query_as!(
                User,
                r#"
        SELECT *
        FROM users
        WHERE id = ?"#,
                id
            )
            .fetch_optional(&mut db)
            .await
            {
                Ok(v)
            } else {
                Err(())
            }
    }
}



