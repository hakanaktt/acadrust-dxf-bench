use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Run the ACadSharp benchmark, preferring a packaged runner beside the exe.
pub fn run_acadsharp(dir: &Path, iterations: usize) -> Result<Output, String> {
    let absolute = dir.canonicalize().map_err(|e| e.to_string())?;
    let mut failures = Vec::new();

    let mut direct = Vec::new();
    if let Ok(value) = env::var("ACADSHARP_RUNNER") {
        direct.push(PathBuf::from(value));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            direct.push(parent.join("acadsharp-bench.exe"));
            direct.push(parent.join("runners/acadsharp-bench.exe"));
        }
    }
    for runner in direct {
        if !runner.is_file() {
            continue;
        }
        match Command::new(&runner)
            .arg("--dir")
            .arg(&absolute)
            .arg("--iterations")
            .arg(iterations.to_string())
            .output()
        {
            Ok(output) if output.status.success() => return Ok(output),
            Ok(_) => failures.push(format!("{}: exited unsuccessfully", runner.display())),
            Err(error) => failures.push(format!("{}: {error}", runner.display())),
        }
    }

    let mut command = Command::new("dotnet");
    command
        .args([
            "run",
            "-c",
            "Release",
            "--project",
            "acadsharp-bench",
            "--",
            "--dir",
        ])
        .arg(&absolute)
        .arg("--iterations")
        .arg(iterations.to_string());
    match command.output() {
        Ok(output) if output.status.success() => Ok(output),
        Ok(output) => {
            let detail = String::from_utf8_lossy(&output.stderr);
            failures.push(format!(
                "dotnet: {}",
                detail.lines().last().unwrap_or("exited unsuccessfully")
            ));
            Err(format!("ACadSharp runner unavailable. Package acadsharp-bench.exe beside the app or install the .NET 8 SDK. Tried: {}", failures.join("; ")))
        }
        Err(error) => {
            failures.push(format!("dotnet: {error}"));
            Err(format!("ACadSharp runner unavailable. Package acadsharp-bench.exe beside the app or install the .NET 8 SDK. Tried: {}", failures.join("; ")))
        }
    }
}
