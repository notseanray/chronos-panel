use std::future::{ready, Ready};

use actix_web::{
    dev::Payload,
    error::{Error as ActixWebError, ErrorUnauthorized},
    http, web, FromRequest, HttpRequest,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::json;

use crate::models::{AppState, TokenClaims};

pub struct AuthenticationGuard {
    pub user_id: String,
}

impl FromRequest for AuthenticationGuard {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req
            .cookie("disco_access_token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            return ready(Err(ErrorUnauthorized(
                json!({"status": "fail", "message": "You are not logged in, please provide token"}),
            )));
        }
        // check if in cache
        // if not in cache try and fetch
        println!("{:?}", token);

        // let data = req.app_data::<web::Data<AppState>>().unwrap();
        //
        // let jwt_secret = data.env.jwt_secret.to_owned();
        // let decode = decode::<TokenClaims>(
        //     token.unwrap().as_str(),
        //     &DecodingKey::from_secret(jwt_secret.as_ref()),
        //     &Validation::new(Algorithm::HS256),
        // );
        unimplemented!();

        // match decode {
        //     Ok(token) => {
                // let user =
                //     futures::executor::block_on(get_halfuser_from_id(&token.claims.sub, data));

                // if user.is_err() {
                //     return ready(Err(ErrorUnauthorized(
                //         json!({"status": "fail", "message": "User belonging to this token no logger exists"}),
                //     )));
                // }
                //
                // ready(Ok(AuthenticationGuard {
                //     user_id: token.claims.sub,
                // }))
        //     }
        //     Err(_) => ready(Err(ErrorUnauthorized(
        //         json!({"status": "fail", "message": "Invalid token or usre doesn't exists"}),
        //     ))),
        // }
    }
}
