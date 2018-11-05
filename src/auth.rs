extern crate jsonwebtoken as jwt;

use self::jwt::{decode, Validation};
use actix_web::FromRequest;
use actix_web;
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config, Error};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: i64,
    sub: String,
}

use actix_web::{Result, HttpRequest};
use actix_web::middleware::{Middleware, Started};

pub struct JWTAuthorizer<T> 
    where T: FnOnce(&HttpRequest) -> &str
{
    secret: String,
    secret_generating_function: T,
}

impl<T: 'static> JWTAuthorizer<T>
    where T: FnOnce(&HttpRequest) -> &str
{
    pub fn new(secret: String, f: T) -> Self {
        JWTAuthorizer {
            secret: secret,
            secret_generating_function: f,
        }
    }
}

impl<S,T: 'static> Middleware<S> for JWTAuthorizer<T>
    where T: Fn(&HttpRequest) -> &str
{
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let path = req.path();
        let secret = match req.method() {
            &actix_web::http::Method::GET => self.secret.to_string(),
            &actix_web::http::Method::POST => self.secret.to_string() + &self.secret.to_string(),
            _ => "".to_string(),
        };
        info!("Secret {}", secret);
        let mut config = Config::default();
        config.realm("Restricted area");
        let auth = BearerAuth::from_request(&req, &config)?;
        // sub == topic
        let validation = Validation {sub: Some("sseplex".into()), ..Default::default()};

        let token_data = match decode::<Claims>(auth.token(), secret.as_ref(), &validation) {
            Ok(c) => c,
            _ => return Err(AuthenticationError::from(config)
                            .with_error(Error::InvalidToken)
                            .into()),
        };
        Ok(Started::Done)
    }
}
