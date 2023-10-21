use clap::{arg, Command};
use directories::ProjectDirs;
use anyhow::Result;

mod app;
use clash::Clash;
use app::App;

fn cli() -> Command {
    Command::new("clash")
        .about("Clash CLI")
        .subcommand(
            Command::new("show")
                .about("Show clash")
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")),
        )
        .subcommand(
            Command::new("next")
                .about("Select next clash")
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")),
        )
        .subcommand(Command::new("status").about("Show status information"))
}

fn main() -> Result<()> {
    let project_dirs =
        ProjectDirs::from("com", "Clash CLI", "clash").expect("Unable to find project directory");

    let app = App::new(project_dirs.data_dir());

    match cli().get_matches().subcommand() {
        Some(("show", args)) => app.show(args),
        Some(("next", args)) => app.next(args),
        Some(("status", args)) => app.status(args),
        _ => Ok(()),
    }
}
