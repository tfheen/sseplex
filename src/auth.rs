extern crate jsonwebtoken as jwt;

use self::jwt::{decode, Validation};
use actix_web::{Result, HttpRequest, FromRequest};
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
    F: FnMut(HttpRequest) -> &'static str
{
    secret_func: F,
}

impl<F> JWTAuthorizer<F>
    where
    F: FnMut(HttpRequest) -> &'static str
{
    pub fn new(f: F) -> Self {
        JWTAuthorizer {
            secret_func: f,
        }
    }
}

impl<S,F: 'static> Middleware<S> for JWTAuthorizer<F> 
    where
    F: FnMut(HttpRequest) -> &'static str,
{
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let sf = self.secret_func;
        let secret = sf(*req);
/*        let path = req.path();
        let secret = match req.method() {
            &actix_web::http::Method::GET => self.secret.to_string(),
            &actix_web::http::Method::POST => self.secret.to_string() + &self.secret.to_string(),
            _ => "".to_string(),
        };*/
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
