use clap::ArgMatches;
use std::path::PathBuf;
use anyhow::{anyhow, Context, Result};
use rand::seq::IteratorRandom;
use crate::Clash;

#[derive(Debug, Clone)]
struct PublicHandle(String);
impl std::fmt::Display for PublicHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct App {
    pub clash_dir: PathBuf,
    pub current_clash_file: PathBuf,
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
        if let Ok(entry) = self
            .clashes()?
            .choose(&mut rng)
            .expect("No clashes to choose from!")
        {
            let filename = entry
                .file_name()
                .into_string()
                .expect("unable to convert OsString to String (?!?)");
            Ok(PublicHandle(match filename.strip_suffix(".json") {
                Some(handle) => handle.to_string(),
                None => filename,
            }))
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

    pub fn show(&self, args: &ArgMatches) -> Result<()> {
        let handle = self.handle_from_args(args).or_else(|_| self.current_handle())?;
        let clash_file = self.clash_dir.join(format!("{}.json", handle));
        let contents = std::fs::read_to_string(clash_file)
            .with_context(|| format!("Unable to find clash with handle {}", handle))?;
        if args.get_flag("raw") {
            println!("{}", &contents);
            return Ok(())
        }
        let clash: Clash = serde_json::from_str(&contents)?;
        // DEBUG
        // dbg!(contents);
        // println!("{}", serde_json::to_string_pretty(&clash).unwrap());
        clash.pretty_print();
        Ok(())
    }

    pub fn next(&self, args: &ArgMatches) -> Result<()> {
        let next_handle = self
            .handle_from_args(args)
            .or_else(|_| self.random_handle())?;
        println!("Changed clash to https://codingame.com/contribute/view/{}", next_handle);
        println!(" Local file: {}/{}.json", &self.clash_dir.to_str().unwrap(), next_handle);
        std::fs::write(&self.current_clash_file, next_handle.to_string())?;
        Ok(())
    }

    pub fn status(&self, _args: &ArgMatches) -> Result<()> {
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

    pub fn run(&self, args: &ArgMatches) -> Result<()> {
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
}

