mod flags;
mod inputs;
mod packages;
mod bars;

use std::process;

use anyhow::{bail, Result};
use dialoguer::Confirm;
use inputs::{Inputs, Secret};
use xshell::{pushd, rm_rf};

use packages::Packages;

const PACKAGES: &[u8] = include_bytes!(concat!(project_root!(), "/assets/packages.yaml"));
const DOTFILES: &str = "https://github.com/oberblastmeister/dotfiles.git";

macro_rules! _cmd {
    ($( $stuff:tt )*) => {{
        use std::process::{Command, Stdio};
        let mut cmd: Command = xshell::cmd!($( $stuff )*).into();
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.spawn()?
    }}
}
pub(crate) use _cmd as cmd;

macro_rules! _cmd_ {
    ($( $stuff:tt )*) => {
        xshell::cmd!($( $stuff )*)
    };
}
pub(crate) use _cmd_ as cmd_;

macro_rules! _project_root {
    () => {
        env!("CARGO_MANIFEST_DIR")
    };
}
pub(crate) use _project_root as project_root;

fn is_root() -> bool {
    sudo::check() == sudo::RunningAs::Root
}

fn install_pkg(pkg: &str) -> Result<()> {
    cmd!("pacman --noconfirm --needed -S {pkg}").wait()?;
    Ok(())
}

fn aur_install(pkg: &str, Inputs { username, .. }: &Inputs) -> Result<()> {
    cmd!("sudo -u {username} yay --noconfirm -S {pkg}").wait()?;
    Ok(())
}

fn manual_install(pkg: &str, inputs: &Inputs) -> Result<()> {
    let _d = pushd("/tmp");
    let _ = rm_rf(pkg);
    let Inputs { username, .. } = inputs;
    cmd!("sudo -u {username} git clone https://aur.archlinux.org/{pkg}.git").wait()?;
    let _d = pushd(pkg);
    cmd!("sudo -u {username} makepkg --noconfirm -si").wait()?;

    Ok(())
}

fn add_user(inputs: &Inputs) -> Result<()> {
    let Inputs { username, password: Secret { inner: password }, .. } = inputs;

    println!("Adding user {}", username);
    cmd!("useradd --create-home --groups wheel --shell /bin/zsh '{username}'").wait()?;
    cmd_!("chpasswd").stdin(format!("{}\n", password)).run()?;

    Ok(())
}

const NEEDED: [&str; 4] = ["sudo", "curl", "base-devel", "git"];

fn install_needed(inputs: &Inputs) -> Result<()> {
    let bar = bars::blue();
    bar.set_length(NEEDED.len() as u64);

    for needed in NEEDED.iter() {
        bar.set_message(format!("Installing {}", needed));
        install_pkg(needed)?;
        bar.inc(1);
    }

    bar.finish_with_message("Finished installing needed packages");

    Ok(())
}

fn clone_dotfiles(dotfiles_url: &str, Inputs { username, .. }: &Inputs) -> Result<()> {
    let bar = bars::loading();
    bar.set_message("Cloning dotfiles");

    cmd!("sudo -u {username} yadm clone {dotfiles_url} --no-bootstrap").wait()?;
    cmd!("sudo -u {username} yadm alt").wait()?;

    bar.finish_with_message("Finished cloneing dotfiles");

    Ok(())
}

fn install_aur_helper(inputs: &Inputs) -> Result<()> {
    let spinner = bars::loading();
    spinner.set_message("Installing aur helper");

    manual_install("yay-bin", inputs)?;

    spinner.finish_with_message("Finished installing aur helper");

    Ok(())
}

fn confirm_install(input: &Inputs) -> Result<bool> {
    println!("These are the inputs that you gave:\n\n{:#?}", input);

    Ok(Confirm::new()
        .with_prompt(
            "Are you sure you want to continue with the install?
This will override everything for this user",
        )
        .interact()?)
}

fn confirm_install2() -> Result<bool> {
    Ok(Confirm::new().with_prompt("Are you really sure you want to install?").interact()?)
}

fn finish() {
    println!("The installation is completed");
}

fn try_main() -> Result<()> {
    if !is_root() {
        bail!("You must be running as root");
    }

    let inputs = Inputs::get()?;

    let packages = Packages::from_slice(PACKAGES)?;

    if !confirm_install(&inputs)? || !confirm_install2()? {
        return Ok(());
    }

    install_needed(&inputs)?;
    install_aur_helper(&inputs)?;
    add_user(&inputs)?;
    packages.install(&inputs)?;
    clone_dotfiles(DOTFILES, &inputs)?;
    finish();

    Ok(())
}

fn main() {
    match try_main() {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("{:?}", e);
            process::exit(1);
        }
    }
}
