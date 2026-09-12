use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Run the ezdxf benchmark with the first usable Python runtime.
///
/// A packaged build can place Python at `python/python.exe` beside the exe,
/// or set `EZDXF_PYTHON`. Developer machines fall back to the Windows
/// launcher (`py -3`) and then PATH-based Python commands.
pub fn run_ezdxf_bench(script: &Path, dir: &Path, iterations: usize) -> Result<Output, String> {
    let mut bundled = Vec::new();
    if let Ok(value) = env::var("EZDXF_RUNNER") {
        bundled.push(PathBuf::from(value));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            bundled.push(parent.join("ezdxf-bench.exe"));
            bundled.push(parent.join("runners/ezdxf-bench.exe"));
        }
    }
    let mut failures = Vec::new();
    for runner in bundled {
        if !runner.is_file() {
            continue;
        }
        match Command::new(&runner)
            .arg("--dir")
            .arg(dir)
            .arg("--iterations")
            .arg(iterations.to_string())
            .output()
        {
            Ok(output) if output.status.success() => return Ok(output),
            Ok(_) => failures.push(format!("{}: exited unsuccessfully", runner.display())),
            Err(error) => failures.push(format!("{}: {error}", runner.display())),
        }
    }

    let mut candidates: Vec<(OsString, Vec<OsString>, String)> = Vec::new();

    if let Ok(value) = env::var("EZDXF_PYTHON") {
        let path = PathBuf::from(value);
        candidates.push((
            path.clone().into_os_string(),
            Vec::new(),
            path.display().to_string(),
        ));
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            for path in [parent.join("python/python.exe"), parent.join("python.exe")] {
                if path.is_file() {
                    candidates.push((
                        path.clone().into_os_string(),
                        Vec::new(),
                        path.display().to_string(),
                    ));
                }
            }
        }
    }

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for path in [
        cwd.join(".venv/Scripts/python.exe"),
        cwd.join(".venv/python.exe"),
    ] {
        if path.is_file() {
            candidates.push((
                path.clone().into_os_string(),
                Vec::new(),
                path.display().to_string(),
            ));
        }
    }

    #[cfg(target_os = "windows")]
    candidates.push((
        OsString::from("py"),
        vec![OsString::from("-3")],
        "py -3".into(),
    ));
    candidates.push((OsString::from("python"), Vec::new(), "python".into()));
    candidates.push((OsString::from("python3"), Vec::new(), "python3".into()));

    for (program, prefix, name) in candidates {
        let mut command = Command::new(&program);
        command
            .args(prefix)
            .arg(script)
            .arg("--dir")
            .arg(dir)
            .arg("--iterations")
            .arg(iterations.to_string());
        match command.output() {
            Ok(output) if output.status.success() => return Ok(output),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail = stderr.lines().last().unwrap_or("exited unsuccessfully");
                failures.push(format!("{name}: {detail}"));
            }
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }

    Err(format!(
        "ezdxf runner unavailable. Install ezdxf==1.4.3 in Python 3 or set EZDXF_PYTHON to a Python executable. Tried: {}",
        failures.join("; ")
    ))
}
