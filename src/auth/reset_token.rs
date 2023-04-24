use anyhow::Error;
use argon2::{Argon2, PasswordHasher};

use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::ThreadRng,
};

pub fn generate() -> Result<(String, String), Error> {
    let mut rng = rand::thread_rng();
    let token = random_string(&mut rng, 32);
    let salt = random_string(&mut rng, 32);
    let argon2 = Argon2::default();
    let pw_hash = argon2.hash_password(token.as_bytes(), &salt).unwrap();
    Ok((token, pw_hash.hash.unwrap().to_string()))
}

fn random_string(rng: &mut ThreadRng, length: usize) -> String {
    Alphanumeric.sample_string(rng, length)
}
