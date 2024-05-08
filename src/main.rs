mod internal;

use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use clashlib::clash::{Clash, PublicHandle, TestCase};
use clashlib::stub::StubConfig;
use clashlib::{solution, stub};
use directories::ProjectDirs;
use internal::OutputStyle;
use rand::seq::IteratorRandom;

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

    Command::new(clap::crate_name!())
        .about("CLI tool for playing CodinGame puzzles and Clash of Code")
        .version(clap::crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("show")
                .about("Show clash")
                .arg(
                    arg!(--"show-whitespace" [BOOL] "render ⏎ and • in place of newlines and spaces")
                        // This means show-whitespace=1 also works
                        .value_parser(clap::builder::BoolishValueParser::new())
                        .default_value("true")
                        .default_missing_value("true")
                )
                .arg(
                    arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")
                        .value_parser(value_parser!(PublicHandle))
                )
                .arg(arg!(-'r' --"reverse" "print the clash in reverse mode"))
        )
        .subcommand(
            Command::new("next")
                .about("Select next clash")
                .arg(
                    arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")
                        .value_parser(value_parser!(PublicHandle))
                        .exclusive(true)
                )
                .arg(arg!(-'r' --"reverse" "pick a random clash that has reverse mode"))
                .arg(arg!(-'s' --"shortest" "pick a random clash that has shortest mode"))
                .arg(arg!(-'f' --"fastest" "pick a random clash that has fastest mode"))
                .after_help(
                    "Pick a random clash from locally stored clashes when PUBLIC_HANDLE is not given.\
                    \nIf instead flags modes are supplied, it will look for a clash that has at least all of those modes available.\
                    \nFor example: coctus next --fastest --shortest will return a clash that has BOTH fastest and shortest as options."
                )
        )
        .subcommand(
            Command::new("run")
                .about("Test a solution against current clash")
                .arg(arg!(--"build-command" <COMMAND> "command that compiles the solution"))
                .arg(arg!(--"command" <COMMAND> "command that executes the solution").required(true))
                .arg(
                    arg!(--"timeout" <SECONDS> "how many seconds before execution is timed out (0 for no timeout)")
                        .value_parser(value_parser!(f64))
                        .default_value("5")
                )
                .arg(arg!(--"auto-advance" "automatically move on to next clash if all test cases pass"))
                .arg(arg!(--"ignore-failures" "run all tests despite failures"))
                .arg(
                    arg!(--"testcases" <TESTCASE_INDICES> "indices of the testcases to run (separated by commas)")
                        .value_parser(value_parser!(u64).range(1..99))
                        .value_delimiter(',')
                )
                .arg(
                    arg!(--"show-whitespace" [BOOL] "render ⏎ and • in place of newlines and spaces")
                        // This means show-whitespace=1 also works
                        .value_parser(clap::builder::BoolishValueParser::new())
                        .default_value("true")
                        .default_missing_value("true")
                )
                .arg(
                    arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")
                        .value_parser(value_parser!(PublicHandle))
                )
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
                .arg(
                    arg!(<PUBLIC_HANDLE> ... "hexadecimal handle of the clash")
                        .value_parser(value_parser!(PublicHandle))
                )
                .after_help(
                    "The PUBLIC_HANDLE of a puzzle is the last part of the URL when viewing it on the contribution section on CodinGame (1).\
                    \nYou can fetch both clash of code and classic (in/out) puzzles.\
                    \n (1) https://www.codingame.com/contribute/community"
                )
        )
        .subcommand(
            Command::new("showtests")
                .about("Print testcases and validators of current clash")
                .arg(
                    arg!(--"show-whitespace" [BOOL] "render ⏎ and • in place of newlines and spaces")
                        // This means show-whitespace=1 also works
                        .value_parser(clap::builder::BoolishValueParser::new())
                        .default_value("false")
                        .default_missing_value("true")
                )
                .arg(arg!(--"in" "only print the testcase input"))
                .arg(arg!(--"out" "only print the testcase output").conflicts_with("in"))
                .arg(
                    arg!([TESTCASE] ... "indices of the testcases to print (default: all)")
                        .value_parser(value_parser!(u64).range(1..99))
                        .value_delimiter(',')
                )
        )
        .subcommand(
            Command::new("json")
                .about("Print the raw source JSON of a clash")
                .arg(
                    arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")
                        .value_parser(value_parser!(PublicHandle))
                )
        )
        .subcommand(
            Command::new("generate-stub")
                .alias("gen")
                .about("Generate input handling code for a given language")
                .arg(arg!(<PROGRAMMING_LANGUAGE> "Programming language of the solution stub"))
                .arg(
                    arg!(--"from-file" <STUBFILE> "Generate stub from a stub generator file instead of the current clash")
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(arg!(--"from-reference" "Generate stub from the reference stub generator instead of the current clash").conflicts_with("from-file"))
                .after_help(
                    "Prints boilerplate code for the input of the current clash.\
                    \nIntended to be piped to a file.\
                    \nExamples:\
                    \n  $ coctus generate-stub ruby > sol.rb\
                    \n  $ coctus generate-stub bash > sol.sh"
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
                    \n  $ coctus generate-shell-completion fish > ~/.config/fish/completions/coctus.fish\
                    \n  $ coctus generate-shell-completion bash >> ~/.config/bash_completion\
                    \n  $ coctus generate-shell-completion powershell >> $PROFILE.CurrentUserCurrentHost\
                    \nNOTE: (powershell) You may need to move the using statements to the top of the script."
                )
        )
}

struct App {
    clash_dir: PathBuf,
    current_clash_file: PathBuf,
    stub_templates_dir: PathBuf,
}

impl App {
    fn new(data_dir: &std::path::Path, config_dir: &std::path::Path) -> App {
        App {
            clash_dir: data_dir.join("clashes"),
            current_clash_file: data_dir.join("current"),
            stub_templates_dir: config_dir.join("stub_templates"),
        }
    }

    // This may fail the very first time we call `show` if `next` was never run.
    fn current_handle(&self) -> Result<PublicHandle> {
        let content = std::fs::read_to_string(&self.current_clash_file)
            .with_context(|| format!("Unable to read {:?}", &self.current_clash_file))?;
        PublicHandle::from_str(&content)
    }

    fn build_stub_config(&self, args: &ArgMatches) -> Result<StubConfig> {
        let lang_arg = args
            .get_one::<String>("PROGRAMMING_LANGUAGE")
            .context("Should have a programming language")?;

        StubConfig::find_stub_config(lang_arg.as_str(), &self.stub_templates_dir)
    }

    fn clashes(&self) -> Result<std::fs::ReadDir> {
        std::fs::read_dir(&self.clash_dir).with_context(|| "No clashes stored")
    }

    fn random_handle(&self) -> Result<PublicHandle> {
        let mut rng = rand::thread_rng();
        if let Ok(entry) = self.clashes()?.choose(&mut rng).expect("No clashes to choose from!") {
            let filename =
                entry.file_name().into_string().expect("unable to convert OsString to String (?!?)");
            PublicHandle::from_str(match filename.strip_suffix(".json") {
                Some(handle) => handle,
                None => &filename,
            })
        } else {
            Err(anyhow!("Unable to randomize next clash"))
        }
    }

    fn random_handle_with_modes(&self, fastest: bool, shortest: bool, reverse: bool) -> Result<PublicHandle> {
        let max_attemps = 100;
        for _ in 0..max_attemps {
            let handle = self.random_handle()?;
            let clash = self.read_clash(&handle)?;
            if (!reverse || clash.is_reverse())
                && (!fastest || clash.is_fastest())
                && (!shortest || clash.is_shortest())
            {
                return Ok(handle)
            }
        }
        Err(anyhow!(format!(
            "Failed to find a clash with the required modes after {} attempts.",
            max_attemps
        )))
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
        let handle = match args.get_one::<PublicHandle>("PUBLIC_HANDLE") {
            Some(h) => h.to_owned(),
            None => self.current_handle()?,
        };
        let clash = self.read_clash(&handle)?;

        let show_whitespace = *args.get_one::<bool>("show-whitespace").unwrap_or(&false);
        let ostyle = OutputStyle::from_env(show_whitespace);

        // --reverse flag
        if args.get_flag("reverse") {
            if clash.is_reverse() {
                ostyle.print_reverse_mode(&clash);
                return Ok(())
            } else {
                return Err(anyhow::Error::msg("The clash doesn't have a reverse mode"))
            }
        }

        // If the clash is reverse only, print the headers and testcases.
        if clash.is_reverse_only() {
            ostyle.print_reverse_mode(&clash);
        } else {
            ostyle.print_headers(&clash);
            ostyle.print_statement(&clash);
        }

        Ok(())
    }

    fn next(&self, args: &ArgMatches) -> Result<()> {
        let next_handle = match args.get_one::<PublicHandle>("PUBLIC_HANDLE") {
            Some(h) => h.to_owned(),
            None => {
                let fastest = args.get_flag("fastest");
                let shortest = args.get_flag("shortest");
                let reverse = args.get_flag("reverse");
                if reverse || fastest || shortest {
                    self.random_handle_with_modes(fastest, shortest, reverse)?
                } else {
                    self.random_handle()?
                }
            }
        };
        println!(" Changed clash to https://codingame.com/contribute/view/{}", next_handle);
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
        let handle = match args.get_one::<PublicHandle>("PUBLIC_HANDLE") {
            Some(h) => h.to_owned(),
            None => self.current_handle()?,
        };

        if let Some(mut build_command) = command_from_argument(args.get_one::<String>("build-command"))? {
            let build = build_command.output()?;

            if !build.status.success() {
                if !build.stderr.is_empty() {
                    println!("Build command STDERR:\n{}", String::from_utf8(build.stderr)?);
                }
                if !build.stdout.is_empty() {
                    println!("Build command STDOUT:\n{}", String::from_utf8(build.stdout)?);
                }
                return Err(anyhow!("Build failed"))
            }
        }

        let mut run_command = command_from_argument(args.get_one::<String>("command"))?
            .expect("clap should ensure `run` can't be executed without a --command");

        let timeout = match *args.get_one::<f64>("timeout").unwrap_or(&5.0) {
            secs if secs.is_nan() => return Err(anyhow!("Timeout can't be NaN")),
            secs if secs < 0.0 => return Err(anyhow!("Timeout can't be negative (use 0 for no timeout)")),
            secs if secs == 0.0 => std::time::Duration::MAX,
            secs => std::time::Duration::from_micros((secs * 1e6) as u64),
        };

        let all_testcases = self.read_clash(&handle)?.testcases().to_owned();

        let testcases: Vec<&TestCase> = if let Some(testcase_indices) = args.get_many::<u64>("testcases") {
            testcase_indices.map(|idx| &all_testcases[(idx - 1) as usize]).collect()
        } else {
            all_testcases.iter().collect()
        };

        let num_tests = testcases.len();
        let suite_run = solution::lazy_run(testcases, &mut run_command, &timeout);

        let ignore_failures = args.get_flag("ignore-failures");
        let show_whitespace = *args.get_one::<bool>("show-whitespace").unwrap_or(&false);
        let ostyle = OutputStyle::from_env(show_whitespace);

        let mut num_passed = 0;

        for (test_case, test_result) in suite_run {
            ostyle.print_result(test_case, &test_result);

            if test_result.is_success() {
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
        std::fs::create_dir_all(&self.clash_dir)?;
        let handles = args
            .get_many::<PublicHandle>("PUBLIC_HANDLE")
            .with_context(|| "Should have many handles")?;
        let http = reqwest::blocking::Client::builder().use_rustls_tls().build()?;
        for handle in handles {
            let res = http
                .post("https://www.codingame.com/services/Contribution/findContribution")
                .body(format!(r#"["{}", true]"#, handle))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()?;
            let content = res.error_for_status()?.text()?;
            let clash_file_path = self.clash_dir.join(format!("{}.json", handle));
            std::fs::write(&clash_file_path, &content)?;
            println!("Saved clash {} as {}", &handle, &clash_file_path.display());
        }
        Ok(())
    }

    fn showtests(&self, args: &ArgMatches) -> Result<()> {
        let handle = self.current_handle()?;
        let clash = self.read_clash(&handle)?;
        let all_testcases = clash.testcases();

        let show_whitespace = *args.get_one::<bool>("show-whitespace").unwrap_or(&false);
        let ostyle = OutputStyle::from_env(show_whitespace);

        let num_testcases = all_testcases.len();
        let testcase_indices: Vec<u64> = match args.get_many::<u64>("TESTCASE") {
            Some(nums) => nums.cloned().collect(),
            None => (1u64..=num_testcases as u64).collect(),
        };

        let only_in = args.get_flag("in");
        let only_out = args.get_flag("out");

        for idx in testcase_indices {
            let testcase = match all_testcases.get((idx - 1) as usize) {
                Some(x) => x,
                None => {
                    return Err(anyhow!(
                    "Invalid testcase index {idx} (the current clash only has {num_testcases} test cases)"
                ))
                }
            };

            if !(only_in || only_out) {
                let styled_title = ostyle.title.paint(format!("#{} {}", idx, testcase.title));
                println!("{styled_title}");
                println!("{}", ostyle.secondary_title.paint("===== INPUT ======"));
            }
            if !only_out {
                println!("{}", ostyle.styled_testcase_input(testcase));
            }
            if !(only_in || only_out) {
                println!("{}", ostyle.secondary_title.paint("==== EXPECTED ===="));
            }
            if !only_in {
                println!("{}", ostyle.styled_testcase_output(testcase));
            }
        }

        Ok(())
    }

    fn generate_stub(&self, args: &ArgMatches) -> Result<()> {
        let config = self.build_stub_config(args)?;

        let stub_generator = match args.get_one::<PathBuf>("from-file") {
            Some(fname) if fname.to_str() == Some("-") => {
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                input
            }
            Some(fname) => std::fs::read_to_string(fname)?,
            None if args.get_flag("from-reference") => stub::SIMPLE_REFERENCE_STUB.to_owned(),
            None => {
                let handle = self.current_handle()?;
                self.read_clash(&handle)?
                    .stub_generator()
                    .with_context(|| "Current clash provides no input stub generator")?
                    .to_owned()
            }
        };

        let stub_string = stub::generate(config, &stub_generator)?;
        println!("{stub_string}");
        Ok(())
    }

    fn json(&self, args: &ArgMatches) -> Result<()> {
        let handle = match args.get_one::<PublicHandle>("PUBLIC_HANDLE") {
            Some(h) => h.to_owned(),
            None => self.current_handle()?,
        };
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
    let project_dirs = ProjectDirs::from("", "CoCtus", "coctus").expect("Unable to find project directory");

    let app = App::new(project_dirs.data_dir(), project_dirs.config_dir());

    match cli().get_matches().subcommand() {
        Some(("show", args)) => app.show(args),
        Some(("next", args)) => app.next(args),
        Some(("status", args)) => app.status(args),
        Some(("run", args)) => app.run(args),
        Some(("fetch", args)) => app.fetch(args),
        Some(("showtests", args)) => app.showtests(args),
        Some(("json", args)) => app.json(args),
        Some(("generate-stub", args)) => app.generate_stub(args),
        Some(("generate-shell-completion", args)) => app.generate_completions(args),
        _ => Err(anyhow!("unimplemented subcommand")),
    }
}
