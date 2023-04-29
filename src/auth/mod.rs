use anyhow::{anyhow, Error};
use bcrypt::{hash, DEFAULT_COST};

pub mod authenticated_user;
pub mod claim;
pub mod token;

pub fn hash_user_password(password: &str) -> Result<String, Error> {
    hash(password, DEFAULT_COST).map_err(|_| anyhow!("Could not hash password."))
}
