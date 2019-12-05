extern crate alcoholic_jwt;
extern crate reqwest;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use serde_json;
use std::env;
use std::io::Read;

#[derive(Debug)]
pub struct JWT(pub serde_json::Value);

impl<'a, 'r> FromRequest<'a, 'r> for JWT {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        if let Ok(oauth_url) = env::var("OAUTH_VALIDATION_URL") {
            let auth: Vec<_> = request.headers().get("Authentication").collect();
            if auth.len() != 1 {
                return Outcome::Forward(());
            }
            let token = auth[0].get(7..).expect("Auth header not in correct format");
            let jwks = get_jwks(&oauth_url).expect("Failed to fetch keys");
            let validations = vec![
                alcoholic_jwt::Validation::Issuer(oauth_url.into()),
                alcoholic_jwt::Validation::NotExpired,
            ];
            let kid = alcoholic_jwt::token_kid(&token)
                .unwrap()
                .expect("Failed to find key by ID");
            let jwk = jwks.find(&kid).expect("Specified key not found in set");

            match alcoholic_jwt::validate(token, jwk, validations) {
                Ok(valid_jwt) => Outcome::Success(JWT(valid_jwt.claims)),
                Err(_) => Outcome::Forward(()),
            }
        } else {
            Outcome::Success(JWT(serde_json::from_str("{}").unwrap()))
        }
    }
}

pub fn get_jwks(url: &str) -> Result<alcoholic_jwt::JWKS, Box<dyn std::error::Error>> {
    let mut oauth_url: String = url.to_owned();
    oauth_url.push_str("/openid/connect/jwks.json");
    let mut res = reqwest::get(&oauth_url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;
    let jwks: alcoholic_jwt::JWKS = serde_json::from_str(&body).expect("Failed to decode JWKS");
    Ok(jwks)
}
