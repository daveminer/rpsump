use crate::auth::claim::Claim;
use crate::config::Settings;
use crate::database::{DbConn, DbPool};
use crate::models::user::User;
use actix_web::{dev, error, http::header::HeaderValue, web, Error, FromRequest, HttpRequest};
use diesel::RunQueryDsl;
use futures::future::err;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use std::future::Future;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: i32,
}

type AuthFuture = <AuthenticatedUser as FromRequest>::Future;

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    #[tracing::instrument(skip(req, _payload))]
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        let user = match user_from_token(auth_header, settings(req)) {
            Ok(user) => user,
            Err(_e) => return unauthorized_err("Invalid token".to_string()),
        };

        let db_pool = req.app_data::<web::Data<DbPool>>().unwrap().get_ref();

        let conn = match db_pool.get() {
            Ok(db_conn) => db_conn,
            Err(_e) => return internal_server_err("Could not get database connection".to_string()),
        };

        let settings = req.app_data::<web::Data<Settings>>().unwrap().get_ref();
        validate_user(user, conn, settings)
    }
}

fn user_from_token(
    auth_header: Option<&HeaderValue>,
    settings: &Settings,
) -> Result<AuthenticatedUser, Error> {
    if auth_header.is_none() {
        return Err(error::ErrorUnauthorized("Missing authentication"));
    };

    let encoded_token = auth_header
        .expect("Could not convert token to string")
        .to_str()
        .unwrap()
        .replace("Bearer ", "");

    parse_token(encoded_token, settings)
}

fn parse_token(token: String, settings: &Settings) -> Result<AuthenticatedUser, Error> {
    match decode::<Claim>(&token, &decoding_key(settings), &Validation::default()) {
        Ok(token) => {
            if token_expired(&token) {
                return Err(error::ErrorUnauthorized("Token expired"));
            }

            Ok(AuthenticatedUser {
                id: token.claims.sub.parse().unwrap(),
            })
        }
        Err(_) => Err(error::ErrorUnauthorized("Invalid token")),
    }
}

fn decoding_key(settings: &Settings) -> DecodingKey {
    DecodingKey::from_secret(settings.jwt_secret.as_ref())
}

fn settings(req: &HttpRequest) -> &Settings {
    req.app_data::<web::Data<Settings>>()
        .expect("Could not get settings")
        .get_ref()
}

fn token_expired(token_expiry: &TokenData<Claim>) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Could not get current time")
        .as_secs();

    token_expiry.claims.exp < now
}

fn validate_user(user: AuthenticatedUser, mut db: DbConn, settings: &Settings) -> AuthFuture {
    let settings_clone = settings.clone();

    Box::pin(async move {
        match User::by_id(user.id).first(&mut db) {
            Ok(user) => validate_activated_status(user, &settings_clone),
            Err(_e) => Err(error::ErrorUnauthorized("Invalid token")),
        }
    })
}

// TODO: change activated for allow list
fn validate_activated_status(
    user: User,
    _settings: &Settings,
) -> Result<AuthenticatedUser, actix_web::Error> {
    Ok(AuthenticatedUser { id: user.id })
    // if user.activated || !settings.user_activation_required {
    //     Ok(AuthenticatedUser { id: user.id })
    // } else {
    //     Err(error::ErrorUnauthorized("User is not active"))
    // }
}

fn unauthorized_err(message: String) -> AuthFuture {
    Box::pin(err(actix_web::error::ErrorUnauthorized(message)))
}

fn internal_server_err(message: String) -> AuthFuture {
    Box::pin(err(actix_web::error::ErrorInternalServerError(message)))
}
