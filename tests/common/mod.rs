use std::process::Command;

pub fn run_binary_with_args(args: &[&str]) -> std::process::Output {
    let mut cmd_args = vec!["run", "--quiet", "--"];
    cmd_args.extend(args);
    Command::new("cargo")
        .args(&cmd_args)
        .output()
        .expect("Failed to execute command")
}

pub fn run_binary() -> std::process::Output {
    run_binary_with_args(&[])
}
