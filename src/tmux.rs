use std::env;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, Context};
use tokio::process::Command;
use tokio::time::timeout;

use crate::types::{AppResult, Pane, Session, Window};

const SESSION_FORMAT: &str = "#{session_id}\x01#{session_name}\x01#{session_windows}\x01#{session_attached}\x01#{session_created}\x01#{session_last_attached}\x01#{session_group}\x01#{session_path}";
const WINDOW_FORMAT: &str =
    "#{window_id}\x01#{session_id}\x01#{window_index}\x01#{window_name}\x01#{window_active}\x01#{pane_current_command}";
const PANE_FORMAT: &str = "#{pane_id}\x01#{window_id}\x01#{session_id}\x01#{pane_index}\x01#{pane_active}\x01#{pane_current_command}\x01#{pane_current_path}";
const DELIMITER: char = '\x01';

pub async fn list_sessions() -> AppResult<Vec<Session>> {
    let output = run_tmux(&["list-sessions", "-F", SESSION_FORMAT]).await?;
    parse_sessions(&output)
}

pub async fn list_windows(session_name: &str) -> AppResult<Vec<Window>> {
    let output = run_tmux(&["list-windows", "-F", WINDOW_FORMAT, "-t", session_name]).await?;
    parse_windows(&output)
}

pub async fn list_panes(target_window: &str) -> AppResult<Vec<Pane>> {
    let output = run_tmux(&["list-panes", "-F", PANE_FORMAT, "-t", target_window]).await?;
    parse_panes(&output)
}

pub async fn create_session(name: &str, path: Option<&str>) -> AppResult<()> {
    let mut args = vec!["new-session", "-d", "-s", name];
    if let Some(path) = path {
        args.extend(["-c", path]);
    }
    run_tmux(&args).await?;
    Ok(())
}

pub async fn kill_session(name: &str) -> AppResult<()> {
    run_tmux(&["kill-session", "-t", name]).await?;
    Ok(())
}

pub async fn rename_session(current_name: &str, new_name: &str) -> AppResult<()> {
    run_tmux(&["rename-session", "-t", current_name, "--", new_name]).await?;
    Ok(())
}

pub async fn switch_client(target_session: &str) -> AppResult<()> {
    run_tmux(&["switch-client", "-t", target_session]).await?;
    Ok(())
}

pub async fn attach_session(target_session: &str) -> AppResult<()> {
    run_tmux(&["attach-session", "-t", target_session]).await?;
    Ok(())
}

pub async fn capture_pane(target_pane: &str) -> AppResult<String> {
    run_tmux(&["capture-pane", "-p", "-t", target_pane]).await
}

pub fn is_inside_tmux() -> bool {
    env::var("TMUX")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

pub async fn has_session(name: &str) -> AppResult<bool> {
    match run_tmux(&["has-session", "-t", name]).await {
        Ok(_) => Ok(true),
        Err(error) => {
            let message = error.to_string();
            if message.contains("can't find session") || message.contains("no server running") {
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

pub async fn run_tmux(args: &[&str]) -> AppResult<String> {
    let command_line = format!("tmux {}", args.join(" "));

    let mut command = Command::new("tmux");
    command.args(args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let output = timeout(Duration::from_secs(5), command.output())
        .await
        .map_err(|_| anyhow!("tmux command timed out after 5 seconds: {command_line}"))?
        .with_context(|| format!("failed to execute {command_line}"))?;

    let stdout = String::from_utf8(output.stdout).unwrap_or_default();
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();

    if output.status.success() {
        return Ok(stdout);
    }

    let status_code = output.status.code().unwrap_or_default();
    let error_text = stderr.trim();
    if error_text.is_empty() {
        Err(anyhow!(
            "tmux command failed ({status_code}): {command_line}"
        ))
    } else {
        Err(anyhow!(
            "tmux command failed ({status_code}): {command_line}: {error_text}"
        ))
    }
}

fn parse_sessions(output: &str) -> AppResult<Vec<Session>> {
    let mut sessions = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let fields = split_fields(line);
        if fields.len() != 8 {
            continue;
        }

        let windows = parse_usize(fields[2]);
        let attached = parse_usize(fields[3]);
        let created = parse_i64_with_empty_default(fields[4], 0);
        let last_attached = parse_i64_with_empty_default(fields[5], 0);

        let (Some(windows), Some(attached), Some(created), Some(last_attached)) =
            (windows, attached, created, last_attached)
        else {
            continue;
        };

        sessions.push(Session {
            id: fields[0].to_string(),
            name: fields[1].to_string(),
            windows,
            attached,
            created,
            last_attached,
            group: optional_field(fields[6]),
            path: fields[7].to_string(),
        });
    }

    Ok(sessions)
}

fn parse_windows(output: &str) -> AppResult<Vec<Window>> {
    let mut windows = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let fields = split_fields(line);
        if fields.len() != 6 {
            continue;
        }

        let index = parse_usize(fields[2]);
        let Some(index) = index else {
            continue;
        };

        windows.push(Window {
            id: fields[0].to_string(),
            session_id: fields[1].to_string(),
            index,
            name: fields[3].to_string(),
            active: fields[4] == "1",
            active_command: fields[5].to_string(),
        });
    }

    Ok(windows)
}

fn parse_panes(output: &str) -> AppResult<Vec<Pane>> {
    let mut panes = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let fields = split_fields(line);
        if fields.len() != 7 {
            continue;
        }

        let index = parse_usize(fields[3]);
        let Some(index) = index else {
            continue;
        };

        panes.push(Pane {
            id: fields[0].to_string(),
            window_id: fields[1].to_string(),
            session_id: fields[2].to_string(),
            index,
            active: fields[4] == "1",
            current_command: fields[5].to_string(),
            current_path: fields[6].to_string(),
        });
    }

    Ok(panes)
}

fn parse_usize(value: &str) -> Option<usize> {
    value.parse().ok()
}

fn parse_i64(value: &str) -> Option<i64> {
    value.parse().ok()
}

fn parse_i64_with_empty_default(value: &str, default: i64) -> Option<i64> {
    if value.is_empty() {
        Some(default)
    } else {
        parse_i64(value)
    }
}

fn optional_field(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn split_fields(line: &str) -> Vec<&str> {
    if line.contains("\\001") {
        line.split("\\001").collect()
    } else {
        line.split(DELIMITER).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sessions() {
        let fixture =
            "$0\x01work\x012\x011\x011770744224\x011770749593\x01\x01/home/aceworks/study\n";
        let sessions = parse_sessions(fixture).expect("fixture should parse");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "$0");
        assert_eq!(sessions[0].name, "work");
    }

    #[test]
    fn test_parse_windows() {
        let fixture = "@0\x01$0\x010\x01editor\x011\x01vim\n";
        let windows = parse_windows(fixture).expect("fixture should parse");
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].id, "@0");
        assert_eq!(windows[0].session_id, "$0");
        assert_eq!(windows[0].name, "editor");
    }

    #[test]
    fn test_parse_panes() {
        let fixture = "%0\x01@0\x01$0\x010\x010\x01bash\x01/home/aceworks/study\n";
        let panes = parse_panes(fixture).expect("fixture should parse");
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].id, "%0");
        assert_eq!(panes[0].window_id, "@0");
        assert_eq!(panes[0].session_id, "$0");
        assert_eq!(panes[0].current_command, "bash");
    }

    #[test]
    fn test_parse_special_chars() {
        let fixture =
            "$1\x01테스트|파이프 with spaces\x011\x010\x011770744224\x011770749593\x01\x01/tmp\n";
        let sessions = parse_sessions(fixture).expect("fixture should parse");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "테스트|파이프 with spaces");
    }

    #[test]
    fn test_parse_empty() {
        let sessions = parse_sessions("").expect("empty parse should succeed");
        let windows = parse_windows("").expect("empty parse should succeed");
        let panes = parse_panes("").expect("empty parse should succeed");

        assert!(sessions.is_empty());
        assert!(windows.is_empty());
        assert!(panes.is_empty());
    }

    #[test]
    fn test_parse_malformed() {
        let fixture = "malformed\n$2\x01valid\x011\x010\x011770744224\x011770749593\x01\x01/tmp\n";
        let sessions = parse_sessions(fixture).expect("fixture should parse");

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "$2");
        assert_eq!(sessions[0].name, "valid");
    }

    #[tokio::test]
    #[ignore]
    async fn test_tmux_integration_special_session_name() {
        let sessions = list_sessions().await.expect("list_sessions should succeed");
        assert!(sessions
            .iter()
            .any(|session| session.name == "테스트|파이프"));

        let exists = has_session("테스트|파이프")
            .await
            .expect("has_session should succeed");
        assert!(exists);
    }
}
