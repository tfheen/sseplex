extern crate jsonwebtoken as jwt;

use self::jwt::{decode, Validation};
use actix_web::{Result, HttpRequest, FromRequest, Request};
use actix_web::middleware::{Middleware, Started};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config, Error};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: i64,
    sub: String,
}

pub struct JWTAuthorizer<F>
    where
    F: Fn(&Request) -> (&str, &str),
{
    handler_func: F,
}

impl<F> JWTAuthorizer<F>
    where
    F: Fn(&Request) -> (&str, &str),
{
    pub fn new(f: F) -> Self {
        JWTAuthorizer {
            handler_func: f,
        }
    }
}

impl<S,F: 'static> Middleware<S> for JWTAuthorizer<F> 
    where
    F: Fn(&Request) -> (&str, &str),
{
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let sf = &self.handler_func;
        let (secret, sub) = sf(req.request());
        info!("Secret {}", secret);
        let mut config = Config::default();
        config.realm("Restricted area");
        let auth = BearerAuth::from_request(&req, &config)?;
        // sub == topic
        let validation = Validation {sub: Some(sub.into()), ..Default::default()};

        match decode::<Claims>(auth.token(), secret.as_ref(), &validation) {
            Ok(_) => Ok(Started::Done),
            _ => Err(AuthenticationError::from(config)
                     .with_error(Error::InvalidToken)
                     .into()),
        }
    }
}
