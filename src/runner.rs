use crate::scenario::{Scenario, SuccessCondition};
use anyhow::{bail, Result};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn run_scenario(scenario: &Scenario, sla_seconds: u64) -> Result<RunResult> {
    println!("\n[on-call] starting environment...");
    compose_up(scenario)?;

    println!("[on-call] injecting fault...");
    run_script(scenario.break_script())?;

    println!("\n{}\n", "─".repeat(60));
    println!("PAGE: {}", scenario.meta.page);
    println!("{}\n", "─".repeat(60));
    println!("SLA: {} minutes", sla_seconds / 60);
    println!("Type your commands. The environment is running.\n");

    let started = Instant::now();
    let deadline = Duration::from_secs(sla_seconds);

    loop {
        if started.elapsed() >= deadline {
            compose_down(scenario)?;
            return Ok(RunResult::Timeout);
        }

        sleep(Duration::from_secs(5)).await;

        if check_success(scenario).await? {
            let elapsed = started.elapsed();
            compose_down(scenario)?;
            return Ok(RunResult::Success { elapsed });
        }
    }
}

async fn check_success(scenario: &Scenario) -> Result<bool> {
    match scenario.meta.success_condition {
        SuccessCondition::Http200 => {
            let url = &scenario.meta.success_target;
            let status = Command::new("curl")
                .args(["-sf", "--max-time", "4", url])
                .status();
            Ok(status.map(|s| s.success()).unwrap_or(false))
        }
        SuccessCondition::ExitZero => {
            let check = scenario.check_script();
            if !check.exists() {
                bail!("check.sh not found at {}", check.display());
            }
            let status = Command::new("bash").arg(check).status();
            Ok(status.map(|s| s.success()).unwrap_or(false))
        }
    }
}

fn compose_up(scenario: &Scenario) -> Result<()> {
    let status = Command::new("docker")
        .args(["compose", "-f"])
        .arg(scenario.compose_file())
        .args(["up", "-d", "--build"])
        .status()?;
    if !status.success() {
        bail!("docker compose up failed");
    }
    Ok(())
}

fn compose_down(scenario: &Scenario) -> Result<()> {
    Command::new("docker")
        .args(["compose", "-f"])
        .arg(scenario.compose_file())
        .args(["down", "-v", "--remove-orphans"])
        .status()
        .ok();
    Ok(())
}

fn run_script(path: std::path::PathBuf) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let status = Command::new("bash").arg(&path).status()?;
    if !status.success() {
        bail!("script {} failed", path.display());
    }
    Ok(())
}

pub enum RunResult {
    Success { elapsed: Duration },
    Timeout,
}
