use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Quick switch"))
        .stdout(predicate::str::contains("use"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("monitor"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("destroy"));
}

#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("usw 0.2.0"));
}

#[test]
fn test_create_help() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("create").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create runtime"));
}

#[test]
fn test_invalid_username() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("create").arg("Invalid_User!")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}

#[test]
fn test_monitor_output() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("monitor")
        .assert()
        .success();
}

#[test]
fn clap_debug_assert() {
    use clap::CommandFactory;
    usw::cli::Cli::command().debug_assert();
}

#[test]
fn test_destroy_help() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("destroy").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroy runtime"));
}

#[test]
fn test_plugin_help() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("plugin").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List plugins"));
}

#[test]
fn test_install_help() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("install").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Install tool"));
}

#[test]
fn test_kill_help() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("kill").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Stop runtime processes"));
}

#[test]
fn test_purge_help() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("purge").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroy runtime"));
}

#[test]
fn test_env_help() {
    let mut cmd = Command::cargo_bin("usw").unwrap();
    cmd.arg("env").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage runtime environment"));
}

#[test]
fn test_subcommand_help() {
    let subs = ["create", "plugin", "install", "monitor", "destroy", "kill", "purge", "env"];
    for sub in &subs {
        let mut cmd = Command::cargo_bin("usw").unwrap();
        cmd.arg(sub).arg("--help").assert().success();
    }
}
