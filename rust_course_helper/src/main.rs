mod checks;

use crate::checks::CHECKS;
use camino::Utf8PathBuf;
use clap::Parser;
use colored::Colorize;
use std::process::ExitCode;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    repo: Utf8PathBuf,
    #[arg(short, long)]
    lab: String,
}

struct Diag {
    text: String,
    path: Option<Utf8PathBuf>,
    help: Option<String>,
}

#[derive(Default)]
struct Diags {
    problems: Vec<Diag>,
}

struct CheckError;

type CheckResult = std::result::Result<(), CheckError>;

impl Diags {
    fn add<S1>(&mut self, text: S1, path: Option<Utf8PathBuf>, help: Option<String>) -> CheckError
    where
        S1: Into<String>,
    {
        self.problems.push(Diag {
            text: text.into(),
            path,
            help: help.map(|x| x.into()),
        });
        CheckError
    }
    fn print(self) {
        if self.problems.is_empty() {
            println!("no problems found");
            return;
        }

        println!("some problems were found:");

        for problem in self.problems {
            println!("{}: {}", "error".red(), problem.text);
            if let Some(path) = problem.path {
                println!("{}: {}", "path".purple(), path);
            }
            if let Some(help) = problem.help {
                println!("{}: {}", "help".blue(), help);
            }
        }
    }
}

fn validate_lab_name(problems: &mut Diags, name: &str) -> CheckResult {
    const NAMES: &[&str] = &[
        "lab01", "lab02", "lab03", "lab04", "lab05", "lab06", "lab07", "project",
    ];
    if !NAMES.contains(&name) {
        let text = format!("`{name}` is not an expected lab name");
        let help = format!("expected one of: {}", NAMES.join(", "));
        return Err(problems.add(text, None, Some(help)));
    }

    Ok(())
}

struct Context<'x> {
    problems: &'x mut Diags,
    repo_path: Utf8PathBuf,
    lab_path: Utf8PathBuf,
}

fn main_impl(problems: &mut Diags) -> CheckResult {
    let args = Args::parse();

    validate_lab_name(problems, &args.lab)?;

    let lab_path = args.repo.join(args.lab);
    let mut context = Context {
        problems,
        repo_path: args.repo,
        lab_path,
    };

    let mut result = Ok(());
    for f in CHECKS {
        let r = f(&mut context);
        result = result.and(r);
    }

    result
}

fn main() -> ExitCode {
    let mut problems = Diags::default();
    let r = main_impl(&mut problems);
    problems.print();

    let (result_text, ret) = match r {
        Ok(_) => ("success".green(), ExitCode::SUCCESS),
        Err(_) => ("failure".red(), ExitCode::FAILURE),
    };
    println!("\nchecker finished with result: {}", result_text);

    ret
}
