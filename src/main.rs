use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use directories::ProjectDirs;
use rand::seq::IteratorRandom;
use std::{path::PathBuf, process::Command};

pub mod clash;
pub mod formatter;
pub mod outputstyle;
pub mod solution;

use clash::Clash;
use outputstyle::OutputStyle;

#[derive(Clone)]
pub enum OutputStyleOption {
    Default,
    Plain
}

fn command_from_argument(cmd_arg: Option<&String>) -> Result<Option<Command>> {
    let cmd = match cmd_arg {
        Some(cmd) => cmd,
        None => return Ok(None),
    };

    match shlex::split(cmd) {
        Some(shlexed_cmd) if shlexed_cmd.is_empty() => Ok(None),
        Some(shlexed_cmd) => {
            let exe = &shlexed_cmd[0];
            let exe_args = &shlexed_cmd[1..];
            let mut cmd = Command::new(exe);
            cmd.args(exe_args);
            Ok(Some(cmd))
        }
        _ => Err(anyhow!("Invalid COMMAND")),
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
                .arg(arg!(--"no-color" "don't use ANSI colors in the output"))
                .arg(
                    arg!(--"show-whitespace" [BOOL] "render ¶ and • in place of newlines and spaces (default: true)")
                        // This means show-whitespace=1 also works
                        .value_parser(clap::builder::BoolishValueParser::new())
                        .default_missing_value("true")
                )
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash"))
                .arg(
                    arg!(-'t' --"testcases" [TESTCASE_NUM] "show the inputs of the testset (shows all if no extra args)")
                        .action(clap::ArgAction::Append)
                        .value_parser(value_parser!(usize))
                )
                .arg(arg!(-'r' --"reverse" "print the clash in reverse mode"))
        )
        .subcommand(
            Command::new("next")
                .about("Select next clash")
                .arg(
                    arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")
                        .exclusive(true)
                )
                .arg(arg!(-'r' --"reverse" "pick a random clash that has reverse mode"))
                .arg(arg!(-'s' --"shortest" "pick a random clash that has shortest mode"))
                .arg(arg!(-'f' --"fastest" "pick a random clash that has fastest mode"))
                .after_help(
                    "Pick a random clash from locally stored clashes when PUBLIC_HANDLE is not given.\
                    \nIf instead flags modes are supplied, it will look for a clash that has at least all of those modes available.\
                    \nFor example: clash next --fastest --shortest will return a clash that has BOTH fastest and shortest as options."
                )
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
            Command::new("json")
                .about("Print the raw source JSON of a clash")
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash"))
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
                    \n  $ clash generate-shell-completion bash >> ~/.config/bash_completion\
                    \n  $ clash generate-shell-completion powershell >> $PROFILE.CurrentUserCurrentHost\
                    \nNOTE: (powershell) You may need to move the using statements to the top of the script."
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
        let clash = self.read_clash(&handle)?;

        let mut ostyle = if args.get_flag("no-color") {
            OutputStyle::plain()
        } else {
            OutputStyle::default()
        };
        if let Some(show_ws) = args.get_one::<bool>("show-whitespace") {
            if *show_ws {
                ostyle.input_whitespace = ostyle.input_whitespace.or(Some(ostyle.input));
                ostyle.output_whitespace = ostyle.output_whitespace.or(Some(ostyle.output));
            } else {
                ostyle.input_whitespace = None;
                ostyle.output_whitespace = None;
            }
        }

        // -t / --testcase flags (temporary)
        if let Some(values) = args.get_many::<usize>("testcases") {
            let testcases_to_print: Vec<usize> = values.cloned().collect();

            // Return an error if any index is out of bounds
            let max_idx = clash.testcases().len() / 2;
            if testcases_to_print.iter().any(|&x| x > max_idx) {
                return Err(anyhow!("Invalid index. The clash only has {} tests.", max_idx));
            }

            // If the flag has no arguments, print everything
            let selection = if testcases_to_print.is_empty() {
                (0..clash.testcases().len()).collect::<Vec<usize>>()
            } else {
                testcases_to_print
            };
            clash.print_testcases(&ostyle, selection);
            return Ok(())
        }

        // --reverse flag
        if args.get_flag("reverse") {
            if clash.is_reverse() {
                clash.print_reverse_mode(&ostyle);
                return Ok(());
            } else {
                return Err(anyhow::Error::msg("The clash doesn't have a reverse mode"));
            }
        }

        // If the clash is reverse only, print the headers and testcases.
        if clash.is_reverse_only() {
            clash.print_reverse_mode(&ostyle);
            return Ok(());
        }

        clash.print_headers(&ostyle);
        clash.print_statement(&ostyle);
        Ok(())
    }

    fn next(&self, args: &ArgMatches) -> Result<()> {
        let next_handle = self
            .handle_from_args(args)
            .or_else(|_| {
                let reverse  = args.get_flag("reverse");
                let fastest  = args.get_flag("fastest");
                let shortest = args.get_flag("shortest");
                if reverse || fastest || shortest {
                    let max_attemps = 100;
                    for _i in 0..max_attemps {
                        let handle = self.random_handle()?;
                        let clash = self.read_clash(&handle)?;
                        if (!reverse  || clash.is_reverse()) 
                        && (!fastest  || clash.is_fastest()) 
                        && (!shortest || clash.is_shortest()) {
                            return Ok(handle);
                        }
                    }
                    Err(anyhow!(format!("Failed to find a clash with the required modes after {} attempts.", max_attemps)))
                } else {
                    self.random_handle()
                }
            })?;
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

        let build_command: Option<Command> = command_from_argument(args.get_one::<String>("build-command"))?;
        solution::build(build_command)?;

        let run_command: Command = command_from_argument(args.get_one::<String>("command"))?
            .expect("--command is required to run solution.");
        let testcases = self.read_clash(&handle)?.testcases().to_owned();
        let num_tests = testcases.len();
        let suite_run = solution::run(testcases, run_command);

        let ignore_failures = args.get_flag("ignore-failures");
        let style = &OutputStyle::default();
        let mut num_passed = 0;

        for test_run in suite_run {
            test_run.print_result(style);

            if test_run.is_successful() {
                num_passed += 1;
            } else if !ignore_failures {
                break
            }
        }
        println!("{num_passed}/{num_tests} tests passed");

        // Move on to next clash if --auto-advance is set
        if num_passed == num_tests && args.get_flag("auto-advance") {
            let next_handle = self.random_handle()?;
            std::fs::write(&self.current_clash_file, next_handle.to_string())?;
            println!("Moving on to next clash...");
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

    fn json(&self, args: &ArgMatches) -> Result<()> {
        let handle = self.handle_from_args(args).or_else(|_| self.current_handle())?;
        let clash_file = self.clash_dir.join(format!("{}.json", handle));
        let contents = std::fs::read_to_string(clash_file)
            .with_context(|| format!("Unable to find clash with handle {}", handle))?;

        println!("{}", &contents);
        Ok(())
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
        Some(("json", args)) => app.json(args),
        Some(("generate-shell-completion", args)) => app.generate_completions(args),
        _ => Err(anyhow!("unimplemented subcommand"))
    }
}
