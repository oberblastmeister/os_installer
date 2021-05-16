use std::{io::{BufRead, BufReader}, process::{self, Child, ChildStderr, ChildStdout}, thread, time::Duration};

use anyhow::{bail, Result};
use dialoguer::Input;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use process::{Command, Stdio};
use xshell::cmd;

// struct Child {
//     stdout: ChildStdout,
//     stderr: ChildStderr,
// }

fn is_root() -> bool {
    sudo::check() == sudo::RunningAs::Root
}

fn install_pkg(pkg: &str) -> Result<Child> {
    let mut cmd: Command = cmd!("pacman --noconfirm --needed -S {pkg}").into();
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    Ok(cmd.spawn()?)
}

fn set_user_and_pass() -> Result<()> {
    let input: String = Input::new().with_prompt("Username").interact()?;

    Ok(())
}

fn get_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-")
}

fn pipe_into(mut child: Child, bar: &ProgressBar) -> Result<()> {
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let stderr = BufReader::new(child.stderr.take().unwrap());

    for line in stdout.lines() {
        bar.set_message(line?);
    }

    child.wait()?;

    Ok(())
}

const NEEDED: [&str; 5] = ["iw", "sudo", "curl", "base-devel", "git"];

fn install_needed_task(bar: ProgressBar) -> Result<()> {
    bar.set_length(NEEDED.len() as u64);

    for needed in NEEDED.iter() {
        install_pkg(needed)?.wait()?;
        bar.set_message(format!("Installing {}", needed));
        bar.inc(1);
    }

    Ok(())
}

pub type TaskFun = Box<dyn FnOnce(ProgressBar) -> Result<()> + Send>;

struct Task {
    fun: TaskFun,
    bar: ProgressBar,
}

impl Task {
    fn new(fun: TaskFun) -> Self {
        let bar = ProgressBar::new(0);
        bar.set_style(get_style());
        Self { fun, bar }
    }

    fn run(self) -> Result<()> {
        (self.fun)(self.bar)
    }
}

struct App {
    style: ProgressStyle,
    progress: MultiProgress,
    tasks: Vec<Task>,
}

impl App {
    fn new() -> App {
        App {
            style: ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .progress_chars("##-"),
            progress: MultiProgress::new(),
            tasks: Vec::new(),
        }
    }

    fn add_task(mut self, task_fun: TaskFun) -> App {
        let mut task = Task::new(task_fun);
        // task.bar = self.progress.add(task.bar);
        self.tasks.push(task);
        self
    }

    fn run(self) -> Result<()> {
        if !is_root() {
            bail!("You must be running as root.");
        }

        for task in self.tasks {
            task.run()?;
        }

        // self.progress.join_and_clear()?;
        self.progress.join()?;

        Ok(())
    }
}

fn main() {
    let res = App::new().add_task(Box::new(install_needed_task)).run();
    match res {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("{:?}", e);
            process::exit(1);
        }
    }
}
