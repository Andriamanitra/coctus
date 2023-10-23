use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use directories::ProjectDirs;
use rand::seq::IteratorRandom;
use std::path::PathBuf;

pub mod clash;
pub mod formatter;
pub mod outputstyle;

use clash::Clash;
use outputstyle::OutputStyle;

pub enum TestRunResult {
    Success,
    WrongOutput {
        stdout: String,
        stderr: String
    },
    RuntimeError {
        stderr: String
    }
}

#[derive(Clone)]
pub enum OutputStyleOption {
    Default,
    Plain
}

pub fn run_test(run: &mut std::process::Command, testcase: &clash::ClashTestCase) -> Result<TestRunResult> {
    let mut run = run
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = run.stdin.as_mut().unwrap();
    std::io::Write::write(&mut stdin, testcase.test_in.as_bytes())?;

    let output = run.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stdout = stdout.trim_end().to_string();
    let stderr = String::from_utf8(output.stderr)?;
    if stdout == testcase.test_out {
        Ok(TestRunResult::Success)
    } else if output.status.success() {
        Ok(TestRunResult::WrongOutput{stdout, stderr})
    } else {
        Ok(TestRunResult::RuntimeError{stderr})
    }
}

fn cli() -> clap::Command {
    use clap::{arg, value_parser, Command};

    Command::new("clash")
        .about("Clash CLI")
        .version(clap::crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("show")
                .about("Show clash")
                // TODO: change these flags to some kind of enum (eg. --style=plain)
                .arg(arg!(--"raw" "do not parse the clash"))
                .arg(arg!(--"no-color" "don't use ANSI colors in the output"))
                .arg(
                    arg!(--"show-whitespace" [BOOL] "render ¶ and • in place of newlines and spaces (default: true)")
                        .value_parser(clap::builder::BoolishValueParser::new())
                        .default_missing_value("true")
                )
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash"))
        )
        .subcommand(
            Command::new("next")
                .about("Select next clash")
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash"))
                .after_help("Picks a random clash from locally stored clashes when PUBLIC_HANDLE is not given.")
        )
        .subcommand(
            Command::new("run")
                .about("Test a solution against current clash")
                .arg(arg!(--"build-command" <COMMAND> "command that compiles the solution"))
                .arg(arg!(--"command" <COMMAND> "command that executes the solution").required(true))
                .arg(arg!(--"auto-advance" "automatically move on to next clash if all test cases pass"))
                .arg(arg!(--"ignore-failures" "run all tests despite failures"))
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash"))
                .after_help(
                    "If a --build-command is specified, it will be executed once before running any of the test cases. \
                    The --command is required and will be executed once per test case.\
                    \nIMPORTANT: The commands you provide will be executed without any sandboxing. Only run code you trust!"
                )
        )
        .subcommand(
            Command::new("status").about("Show status information")
        )
        .subcommand(
            Command::new("fetch")
                .about("Fetch a clash from codingame.com and save it locally")
                .arg(arg!(<PUBLIC_HANDLE> ... "hexadecimal handle of the clash"))
                .after_help(
                    "The PUBLIC_HANDLE of a puzzle is the last part of the URL when viewing it on the contribution section on CodinGame (1).\
                    \nYou can fetch both clash of code and classic (in/out) puzzles.\
                    \n (1) https://www.codingame.com/contribute/community"
                )
        )
        .subcommand(
            Command::new("generate-shell-completion")
                .about("Generate shell completion")
                .arg(arg!(<SHELL>).value_parser(value_parser!(clap_complete::Shell)))
                .after_help(
                    "Prints shell completion for the selected shell to stdout.\
                    \nIntended to be piped to a file. See documentation for your shell for details about where to place the completion file.\
                    \nExamples:\
                    \n  $ clash generate-shell-completion fish > ~/.config/fish/completions/clash.fish\
                    \n  $ clash generate-shell-completion bash >> ~/.config/bash_completion"
                )
        )
}

#[derive(Debug, Clone)]
struct PublicHandle(String);
impl PublicHandle {
    fn new(s: &str) -> Result<PublicHandle> {
        if s.chars().all(|ch| ch.is_ascii_hexdigit()) {
            Ok(PublicHandle(String::from(s)))
        } else {
            Err(anyhow!("Invalid clash handle '{}' (valid handles only contain characters 0-9 and a-f)", s))
        }
    }
}
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
        PublicHandle::new(&content)
    }

    fn handle_from_args(&self, args: &ArgMatches) -> Result<PublicHandle> {
        match args.get_one::<String>("PUBLIC_HANDLE") {
            Some(s) => PublicHandle::new(s),
            None => Err(anyhow!("No clash handle given")),
        }
    }

    fn clashes(&self) -> Result<std::fs::ReadDir> {
        std::fs::read_dir(&self.clash_dir).with_context(|| "No clashes stored")
    }

    fn random_handle(&self) -> Result<PublicHandle> {
        let mut rng = rand::thread_rng();
        if let Ok(entry) = self
            .clashes()?
            .choose(&mut rng)
            .expect("No clashes to choose from!")
        {
            let filename = entry
                .file_name()
                .into_string()
                .expect("unable to convert OsString to String (?!?)");
            PublicHandle::new(match filename.strip_suffix(".json") {
                Some(handle) => handle,
                None => &filename,
            })
        } else {
            Err(anyhow!("Unable to randomize next clash"))
        }
    }

    fn read_clash(&self, handle: &PublicHandle) -> Result<Clash> {
        let clash_file = self.clash_dir.join(format!("{}.json", handle));
        let contents = std::fs::read_to_string(&clash_file)
            .with_context(|| format!("Unable to find clash with handle {}", handle))?;
        let clash: Clash = serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize clash from {:?}", &clash_file))?;
        Ok(clash)
    }

    fn show(&self, args: &ArgMatches) -> Result<()> {
        let handle = self.handle_from_args(args).or_else(|_| self.current_handle())?;
        let clash_file = self.clash_dir.join(format!("{}.json", handle));
        let contents = std::fs::read_to_string(clash_file)
            .with_context(|| format!("Unable to find clash with handle {}", handle))?;
        if args.get_flag("raw") {
            println!("{}", &contents);
            return Ok(())
        }
        let mut styles = if args.get_flag("no-color") {
            OutputStyle::plain()
        } else {
            OutputStyle::default()
        };
        if let Some(show_ws) = args.get_one::<bool>("show-whitespace") {
            if *show_ws {
                styles.input_whitespace = styles.input_whitespace.or(Some(styles.input));
                styles.output_whitespace = styles.output_whitespace.or(Some(styles.output));
            } else {
                styles.input_whitespace = None;
                styles.output_whitespace = None;
            }
        }
        let clash: Clash = serde_json::from_str(&contents)?;

        clash.pretty_print(styles)
    }

    fn next(&self, args: &ArgMatches) -> Result<()> {
        let next_handle = self
            .handle_from_args(args)
            .or_else(|_| self.random_handle())?;
        println!("Changed clash to https://codingame.com/contribute/view/{}", next_handle);
        println!(" Local file: {}/{}.json", &self.clash_dir.to_str().unwrap(), next_handle);
        std::fs::write(&self.current_clash_file, next_handle.to_string())?;
        Ok(())
    }

    fn status(&self, _args: &ArgMatches) -> Result<()> {
        println!("Current clash file: {}", self.current_clash_file.display());
        match self.current_handle() {
            Ok(handle) => println!("Current clash: {}", handle),
            Err(_) => println!("Current clash: -"),
        }
        println!("Clash dir: {}", self.clash_dir.display());
        let num_clashes = match self.clashes() {
            Ok(clashes) => clashes.count(),
            Err(_) => 0,
        };
        println!("Number of clashes: {}", num_clashes);
        Ok(())
    }

    fn run(&self, args: &ArgMatches) -> Result<()> {
        let handle = self
            .handle_from_args(args)
            .or_else(|_| self.current_handle())?;

        // Run build
        if let Some(build_cmd) = args.get_one::<String>("build-command") {
            match shlex::split(build_cmd) {
                Some(shlexed_build_cmd) if shlexed_build_cmd.len() > 0 => {
                    let exe = &shlexed_build_cmd[0];
                    let exe_args = &shlexed_build_cmd[1..];
                    let build = std::process::Command::new(exe).args(exe_args).output().with_context(|| format!("Unable to run build-command '{}'", exe))?;
                    if !build.status.success() {
                        if build.stderr.len() > 0 {
                            println!("Build command STDERR:\n{}", String::from_utf8(build.stderr)?);
                        }
                        if build.stdout.len() > 0 {
                            println!("Build command STDOUT:\n{}", String::from_utf8(build.stdout)?);
                        }
                        return Err(anyhow!("Build failed"));
                    }
                }
                _ => return Err(anyhow!("Invalid --build-command")),
            };
        }

        // Run tests
        if let Some(run_cmd) = args.get_one::<String>("command") {
            match shlex::split(run_cmd) {
                Some(shlexed_run_cmd) if shlexed_run_cmd.len() > 0 => {
                    let exe = &shlexed_run_cmd[0];
                    let exe_args = &shlexed_run_cmd[1..];
                    let mut run = std::process::Command::new(exe);
                    let run = run.args(exe_args);
                    let clash = self.read_clash(&handle)?;
                    let ignore_failures = args.get_flag("ignore-failures");
                    let mut num_passed = 0;
                    let total = clash.testcases().len();
                    for testcase in clash.testcases() {
                        match run_test(run, testcase)? {
                            TestRunResult::Success => { num_passed += 1; },
                            TestRunResult::WrongOutput{stderr, stdout} => {
                                if stderr.len() > 0 {
                                    println!("stderr   : {}", stderr);
                                }
                                println!("{}", testcase.title);
                                println!("==== EXPECTED ==\n{}", testcase.test_out);
                                println!("===== ACTUAL ===\n{}", stdout);
                                if !ignore_failures {
                                    break;
                                }
                            },
                            TestRunResult::RuntimeError{stderr} => {
                                println!("{}", stderr);
                                if !ignore_failures {
                                    break;
                                }
                            },
                        }
                    }
                    if num_passed == total {
                        println!("{}/{} tests passed!", num_passed, total);
                        // Move on to next clash if --auto-advance is set
                        if args.get_flag("auto-advance") {
                            let next_handle = self.random_handle()?;
                            std::fs::write(&self.current_clash_file, next_handle.to_string())?;
                            println!("Moving on to next clash...");
                        }
                    } else {
                        println!("{}/{} tests passed", num_passed, total);
                    }
                }
                _ => return Err(anyhow!("Invalid --command")),
            }
        }

        Ok(())
    }

    fn fetch(&self, args: &ArgMatches) -> Result<()> {
        if let Some(handles) = args.get_many::<String>("PUBLIC_HANDLE") {
            for handle in handles {
                let handle = PublicHandle::new(handle)?;
                let http = reqwest::blocking::Client::new();
                let res = http.post("https://www.codingame.com/services/Contribution/findContribution")
                    .body(format!(r#"["{}", true]"#, handle))
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .send()?;
                let content = res.error_for_status()?.text()?;
                let clash_file_path = self.clash_dir.join(format!("{}.json", handle));
                std::fs::write(&clash_file_path, &content)?;
                println!("Saved clash {} as {}", &handle, &clash_file_path.display());
            }
            Ok(())
        } else {
            Err(anyhow!("fetched no clashes"))
        }
    }

    fn generate_completions(&self, args: &ArgMatches) -> Result<()> {
        let generator = args
            .get_one::<clap_complete::Shell>("SHELL")
            .copied()
            .with_context(|| anyhow!("shell required"))?;
        let mut cmd = cli();
        let name = String::from(cmd.get_name());
        eprintln!("Generating {generator} completions...");
        clap_complete::generate(generator, &mut cmd, name, &mut std::io::stdout());
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
        Some(("run", args)) => app.run(args),
        Some(("fetch", args)) => app.fetch(args),
        Some(("generate-shell-completion", args)) => app.generate_completions(args),
        _ => Err(anyhow!("unimplemented subcommand"))
    }
}
