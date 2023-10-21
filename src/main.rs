use clap::{arg, Command};
use directories::ProjectDirs;

mod app;
use clash::Clash;
use app::App;

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

fn cli() -> Command {
    Command::new("clash")
        .about("Clash CLI")
        .version(clap::crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("show")
                .about("Show clash")
                .arg(arg!(--"raw" "do not parse the clash"))
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash"))
        )
        .subcommand(
            Command::new("next")
                .about("Select next clash")
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash")),
        )
        .subcommand(
            Command::new("run")
                .about("Test a solution against current clash")
                .arg(arg!(--"build-command" <COMMAND> "command that compiles the solution"))
                .arg(arg!(--"command" <COMMAND> "command that executes the solution").required(true))
                .arg(arg!(--"auto-advance" "automatically move on to next clash if all test cases pass"))
                .arg(arg!(--"ignore-failures" "run all tests despite failures"))
                .arg(arg!([PUBLIC_HANDLE] "hexadecimal handle of the clash"))
        )
        .subcommand(Command::new("status").about("Show status information"))
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
        _ => Err(anyhow!("unimplemented subcommand"))
    }
}
