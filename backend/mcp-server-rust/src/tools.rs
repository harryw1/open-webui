use anyhow::{Result, anyhow};
use std::fs;
use std::process::Command;

pub fn read_file(path: String) -> Result<String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read file '{}': {}", path, e))?;
    Ok(content)
}

pub fn list_directory(path: String) -> Result<Vec<String>> {
    let entries = fs::read_dir(&path)
        .map_err(|e| anyhow!("Failed to read directory '{}': {}", path, e))?
        .map(|res| res.map(|e| e.path().display().to_string()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|e| anyhow!("Failed to collect entries: {}", e))?;
    Ok(entries)
}

pub fn shell_command(cmd: String) -> Result<String> {
    let whitelist = vec!["ls", "cat", "grep", "pwd", "echo", "find", "whoami"];

    // Simple parsing to check the command program
    let parts: Vec<String> = shlex::split(&cmd)
        .ok_or_else(|| anyhow!("Failed to parse command"))?;

    if parts.is_empty() {
        return Ok("Empty command".to_string());
    }

    let program = &parts[0];
    if !whitelist.contains(&program.as_str()) {
        return Ok(format!("Command '{}' is not allowed.", program));
    }

    let mut command = Command::new(program);
    command.args(&parts[1..]);

    let output = command.output()
        .map_err(|e| anyhow!("Failed to execute command: {}", e))?;

    let mut result = String::new();
    if !output.stdout.is_empty() {
        result.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !result.is_empty() {
            result.push_str("\n--- stderr ---\n");
        }
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    Ok(result)
}
