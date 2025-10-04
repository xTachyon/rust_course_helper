use crate::{CheckResult, Context};
use std::{
    fs,
    process::{Command, ExitStatus},
};

pub type CheckFn = fn(ctx: &mut Context) -> CheckResult;

pub const CHECKS: &[CheckFn] = &[check_gitignore, check_commited_files];

fn check_gitignore(ctx: &mut Context) -> CheckResult {
    let gitignore_path = ctx.repo_path.join(".gitignore");
    let help = "you need to have a file like this: https://github.com/xTachyon/rust_course_helper/blob/main/.gitignore";

    if !gitignore_path.exists() {
        return Err(ctx.problems.add(
            ".gitignore doesn't exist",
            Some(gitignore_path),
            Some(help.into()),
        ));
    }

    let Ok(text) = fs::read_to_string(&gitignore_path) else {
        return Err(ctx
            .problems
            .add("can't read file", Some(gitignore_path), None));
    };

    if !text.lines().any(|x| x.contains("target")) {
        return Err(ctx.problems.add(
            "target folder doesn't exist in .gitignore",
            Some(gitignore_path),
            Some(help.into()),
        ));
    }

    Ok(())
}

fn command_check_return(ctx: &mut Context, name: &str, e: ExitStatus) -> CheckResult {
    if !e.success() {
        return Err(ctx.problems.add(
            format!("command `{name}` failed: {e}"),
            Some(ctx.repo_path.clone()),
            None,
        ));
    }
    Ok(())
}

fn check_commited_files(ctx: &mut Context) -> CheckResult {
    let output = match Command::new("git")
        .arg("ls-files")
        .current_dir(&ctx.repo_path)
        .output()
    {
        Ok(x) => x,
        Err(e) => {
            return Err(ctx.problems.add(
                format!("git failed: {e}"),
                Some(ctx.repo_path.clone()),
                None,
            ));
        }
    };
    command_check_return(ctx, "git", output.status)?;

    let stdout = String::from_utf8(output.stdout).expect("from_utf8 failed.. somehow");

    const EXTENSIONS: &[&str] = &[
        ".exe", ".dll", ".pdb", ".lib", ".obj", ".so", ".dylib", ".a", ".o", ".rlib", ".rmeta",
        ".d",
    ];

    let mut bad_files = Vec::new();
    for line in stdout.lines() {
        for ext in EXTENSIONS {
            if line.ends_with(ext) {
                bad_files.push(line);
                break;
            }
        }
    }

    if bad_files.is_empty() {
        return Ok(());
    }

    let max_lines = 20;
    let first_bad_files = &bad_files[..bad_files.len().min(max_lines)];
    let mut text = format!(
        "build files were found in the repo. bad files: {}",
        first_bad_files.join("\n")
    );
    if bad_files.len() > max_lines {
        text += format!("\n..and {} more", bad_files.len() - max_lines).as_str();
    }

    Err(ctx.problems.add(
        text,
        Some(ctx.repo_path.clone()),
        Some("remove target directories and all build artifacts".into()),
    ))
}
