use core::fmt;

use anyhow::Result;
use dialoguer::{Input, Password};

pub struct Secret<T> {
    pub inner: T,
}

impl<T> Secret<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret")
    }
}

#[derive(Debug)]
pub struct Inputs {
    pub username: String,
    // pub hostname: String,
    // pub password: Secret<String>,
    // pub root_password: Secret<String>,
}

impl Inputs {
    pub fn get() -> Result<Inputs> {
        const MISMATCH_ERR: &str = "The passwords did not match";

        println!("I need some inputs before we start the install");

        let username = Input::new().with_prompt("Username").interact()?;

        Ok(Self { username })
    }
}
