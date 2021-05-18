use indicatif::{ProgressBar, ProgressStyle};

pub fn blue() -> ProgressBar {
    let bar = ProgressBar::new(0);
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");
    bar.set_style(style);
    bar
}

pub fn loading() -> ProgressBar {
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style);
    spinner
}
