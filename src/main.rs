use anyhow::{anyhow, Context, Result};
use clap::{arg, ArgMatches, Command};
use directories::ProjectDirs;
use rand::seq::IteratorRandom;
use std::path::PathBuf;
use serde_json;

use clash::Clash;

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

#[derive(Debug, Clone)]
struct PublicHandle(String);
impl std::fmt::Display for PublicHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct App {
    clash_dir: PathBuf,
    current_clash_file: PathBuf,
}

impl App {
    fn new(data_dir: &std::path::Path) -> App {
        App {
            clash_dir: data_dir.join("clashes"),
            current_clash_file: data_dir.join("current"),
        }
    }

    fn current_handle(&self) -> Result<PublicHandle> {
        let content = std::fs::read_to_string(&self.current_clash_file)
            .with_context(|| format!("Unable to read {:?}", &self.current_clash_file))?;
        Ok(PublicHandle(content))
    }

    fn handle_from_args(&self, args: &ArgMatches) -> Result<PublicHandle> {
        match args.get_one::<String>("PUBLIC_HANDLE") {
            Some(s) => Ok(PublicHandle(s.to_owned())),
            None => Err(anyhow!("No clash handle given")),
        }
    }

    fn clashes(&self) -> Result<std::fs::ReadDir> {
        std::fs::read_dir(&self.clash_dir).with_context(|| "No clashes stored")
    }

    fn random_handle(&self) -> Result<PublicHandle> {
        let mut rng = rand::thread_rng();
        if let Ok(entry) = self.clashes()?.choose(&mut rng).expect("No clashes to choose from!") {
            let filename = entry.file_name().into_string().expect("unable to convert OsString to String (?!?)");
            Ok(PublicHandle(match filename.strip_suffix(".json") {
                Some(handle) => handle.to_string(),
                None => filename,
            }))
        } else {
            Err(anyhow!("Unable to randomize next clash"))
        }
    }

    fn show(&self, args: &ArgMatches) -> Result<()> {
        let handle = self.handle_from_args(args).or_else(|_| self.current_handle())?;
        println!("https://www.codingame.com/contribute/view/{}", handle);
        let clash_file = self.clash_dir.join(format!("{}.json", handle));
        let contents = std::fs::read_to_string(clash_file)
        .with_context(|| format!("Unable to find clash with handle {}", handle))?;
        let clash: Clash = serde_json::from_str(&contents)?;
        // DEBUG
        // dbg!(contents);
        // println!("{}", serde_json::to_string_pretty(&clash).unwrap());
        clash.pretty_print();
        Ok(())
    }

    fn next(&self, args: &ArgMatches) -> Result<()> {
        let next_handle = self
            .handle_from_args(args)
            .or_else(|_| self.random_handle())?;
        println!("{:?}", next_handle);
        std::fs::write(&self.current_clash_file, next_handle.to_string())?;
        Ok(())
    }

    fn status(&self, _args: &ArgMatches) -> Result<()> {
        println!("Current clash file: {}", self.current_clash_file.display());
        match self.current_handle() {
            Ok(handle) => println!("Current clash: {}", handle),
            _ => println!("Current clash: -"),
        }
        println!("Clash dir: {}", self.clash_dir.display());
        println!("Number of clashes: {}", self.clashes()?.count());
        Ok(())
    }
}

fn main() -> Result<()> {
    // We look for the locally stored clashes here:
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
