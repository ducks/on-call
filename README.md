# on-call

A terminal game where you get paged and have to fix real broken infrastructure to win. Each round is an incident. The environment is Docker. The tools are yours.

Built as a training tool - every session is recorded as a labeled dataset of broken states, diagnostic command sequences, and resolution steps.

## How it works

1. You run a scenario
2. A broken environment spins up in Docker
3. You get a page - a fake alert describing the symptoms
4. You diagnose and fix it using real tools (`docker logs`, `curl`, `psql`, whatever)
5. The engine polls a health check in the background
6. When it goes green, you win

## Install

```bash
git clone https://github.com/ducks/on-call
cd on-call
nix-shell  # or: cargo build --release
```

Requires Docker.

## Usage

```bash
# list available scenarios
cargo run -- list

# run a scenario
cargo run -- run 001-nginx-502

# run with a shorter SLA
cargo run -- run 001-nginx-502 --sla 5

# export session records as JSONL
cargo run -- export
```

## Scenarios

| ID | Title | Difficulty |
|----|-------|------------|
| 001-nginx-502 | 502 Bad Gateway | 1 |
| 002-postgres-wont-start | Postgres Won't Start | 1 |
| 003-missing-env-var | App Crashing on Boot | 1 |
| 004-disk-full | Disk Full | 2 |
| 005-oom-kill | Container Keeps Restarting | 2 |

## Training data

Sessions are recorded to `~/.local/share/on-call/sessions/sessions.jsonl`. Export with:

```bash
cargo run -- export > sessions.jsonl
```

Each record contains the scenario ID, outcome, time to resolve, and hints used.

## Adding scenarios

Each scenario is a directory under `scenarios/` with:

```
scenarios/my-scenario/
  meta.json            # title, page text, difficulty, hints, success condition
  docker-compose.yml   # the environment (working state)
  break.sh             # injected after compose up to introduce the fault
  check.sh             # polled every 5s to detect resolution (or use http_200)
```

See `SPEC.md` for the full format and `scenarios/001-nginx-502/` for a working example.
