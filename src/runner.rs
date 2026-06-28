use crate::scenario::{Scenario, SuccessCondition};
use anyhow::{bail, Result};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

const HINT_FLAG: &str = "/tmp/on-call-hint";

pub async fn run_scenario(scenario: &Scenario, sla_seconds: u64) -> Result<RunResult> {
    println!("\n[on-call] starting environment...");
    compose_up(scenario)?;

    println!("[on-call] injecting fault...");
    run_script(scenario.break_script())?;

    let container = primary_container(scenario)?;
    inject_hint_command(&container)?;

    let hr: String = "─".repeat(60);
    println!("\n{hr}");
    println!("PAGE: {}", scenario.meta.page);
    println!("{hr}");
    println!("SLA: {} minutes", sla_seconds / 60);
    if !scenario.meta.hints.is_empty() {
        println!("Hints available: {} (run get-hint inside the shell)", scenario.meta.hints.len());
    }
    println!();

    let solved = Arc::new(AtomicBool::new(false));
    let timed_out = Arc::new(AtomicBool::new(false));
    let hints_used = Arc::new(AtomicUsize::new(0));

    let solved_bg = solved.clone();
    let timed_out_bg = timed_out.clone();
    let hints_used_bg = hints_used.clone();
    let scenario_dir = scenario.dir.clone();
    let check_script = scenario.check_script();
    let success_condition = scenario.meta.success_condition.clone();
    let success_target = scenario.meta.success_target.clone();
    let deadline = Duration::from_secs(sla_seconds);
    let hints = scenario.meta.hints.clone();
    let container_bg = container.clone();

    let poller = tokio::task::spawn_blocking(move || {
        let started = Instant::now();
        loop {
            std::thread::sleep(Duration::from_secs(2));

            if started.elapsed() >= deadline {
                timed_out_bg.store(true, Ordering::SeqCst);
                return;
            }

            // Check for hint request
            let hint_flag_exists = Command::new("docker")
                .args(["exec", &container_bg, "test", "-f", HINT_FLAG])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if hint_flag_exists {
                // Remove the flag
                Command::new("docker")
                    .args(["exec", &container_bg, "rm", "-f", HINT_FLAG])
                    .status()
                    .ok();

                let idx = hints_used_bg.fetch_add(1, Ordering::SeqCst);
                if let Some(hint) = hints.get(idx) {
                    println!("\n[hint {}] {}\n", idx + 1, hint);
                } else {
                    println!("\n[no more hints available]\n");
                    // Don't increment past the end
                    hints_used_bg.fetch_sub(1, Ordering::SeqCst);
                }
            }

            // Check success condition
            let ok = match success_condition {
                SuccessCondition::Http200 => Command::new("curl")
                    .args(["-sf", "--max-time", "4", &success_target])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false),
                SuccessCondition::ExitZero => {
                    if check_script.exists() {
                        Command::new("bash")
                            .arg(&check_script)
                            .current_dir(&scenario_dir)
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
            };

            if ok {
                solved_bg.store(true, Ordering::SeqCst);
                return;
            }
        }
    });

    let started = Instant::now();
    Command::new("docker")
        .args([
            "exec",
            "-it",
            &container,
            "sh",
            "-c",
            "[ -x /bin/bash ] && exec /bin/bash || exec /bin/sh",
        ])
        .status()
        .ok();

    poller.abort();

    let elapsed = started.elapsed();
    let used = hints_used.load(Ordering::SeqCst);

    compose_down(scenario)?;

    if solved.load(Ordering::SeqCst) {
        return Ok(RunResult::Success { elapsed, hints_used: used });
    }
    if timed_out.load(Ordering::SeqCst) {
        return Ok(RunResult::Timeout { hints_used: used });
    }
    Ok(RunResult::Abandoned)
}

fn inject_hint_command(container: &str) -> Result<()> {
    // Write a tiny script to /usr/local/bin/get-hint so it works in both bash and sh
    let script = "#!/bin/sh\ntouch /tmp/on-call-hint\necho 'Hint requested...'";
    Command::new("docker")
        .args([
            "exec",
            container,
            "sh",
            "-c",
            &format!("printf '{}' > /usr/local/bin/get-hint && chmod +x /usr/local/bin/get-hint", script),
        ])
        .status()
        .ok();
    Ok(())
}

fn primary_container(scenario: &Scenario) -> Result<String> {
    let output = Command::new("docker")
        .args(["compose", "-f"])
        .arg(scenario.compose_file())
        .args(["ps", "-q"])
        .output()?;

    let ids: Vec<&str> = std::str::from_utf8(&output.stdout)?
        .lines()
        .filter(|l| !l.is_empty())
        .collect();

    let target = scenario.meta.shell_service.as_deref().unwrap_or("");

    if !target.is_empty() {
        let name_output = Command::new("docker")
            .args(["compose", "-f"])
            .arg(scenario.compose_file())
            .args(["ps", "-q", target])
            .output()?;
        let id = std::str::from_utf8(&name_output.stdout)?.trim().to_string();
        if !id.is_empty() {
            return Ok(id);
        }
    }

    ids.first()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("no containers found"))
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
    Success { elapsed: Duration, hints_used: usize },
    Timeout { hints_used: usize },
    Abandoned,
}
