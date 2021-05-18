use crate::{cmd, cmd_, inputs::Inputs};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{aur_install, bars, install_pkg};

#[derive(Debug, Serialize, Deserialize)]
pub struct Packages {
    pub packages: Vec<Package>,
    pub services: Vec<Service>,
}

impl Packages {
    pub fn from_slice(slice: &[u8]) -> Result<Packages> {
        Ok(serde_yaml::from_slice(slice)?)
    }

    pub fn validate(&self) -> Result<()> {
        for package in &self.packages {
            package.validate()?;
        }

        Ok(())
    }

    pub fn install(&self, inputs: &Inputs) -> Result<()> {
        let Packages { packages, .. } = self;

        let bar = bars::blue();
        bar.set_length(packages.len() as u64);

        for package in packages {
            let Package { name, .. } = package;

            bar.set_message(format!("Installing package: {}", name));
            package.install(inputs)?;
            bar.inc(1);
        }

        bar.finish_with_message("Finished installing all packages");

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,

    #[serde(default = "Default::default")]
    pub aur: bool,
}

impl Package {
    fn validate(&self) -> Result<()> {
        let Package { name, .. } = self;
        if cmd!("yay -Ss {name}").wait()?.success() {
            bail!("Package {:?} does not exist", self);
        }
        Ok(())
    }

    fn install(&self, inputs: &Inputs) -> Result<()> {
        let Package { name, aur, .. } = self;
        if *aur {
            aur_install(name, inputs);
        } else {
            install_pkg(name);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
}
