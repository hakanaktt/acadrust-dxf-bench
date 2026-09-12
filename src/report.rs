use serde::ser::Serializer;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// One benchmark row. Non-applicable or failed timings are emitted as null in JSON.
#[derive(Clone, Debug, Serialize)]
pub struct TimingResult {
    pub label: String,
    #[serde(serialize_with = "serialize_ms")]
    pub dxf_ms: f64,
    #[serde(serialize_with = "serialize_ms")]
    pub acadrust_ms: f64,
    #[serde(serialize_with = "serialize_ms")]
    pub acadsharp_ms: f64,
    #[serde(serialize_with = "serialize_ms")]
    pub ezdxf_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReportSection {
    pub name: String,
    pub results: Vec<TimingResult>,
}

impl ReportSection {
    pub fn new(name: impl Into<String>, results: Vec<TimingResult>) -> Self {
        Self {
            name: name.into(),
            results,
        }
    }
}

#[derive(Debug, Serialize)]
struct ReportDocument<'a> {
    generated_at_unix: u64,
    mode: &'a str,
    iterations: usize,
    sections: &'a [ReportSection],
}

fn serialize_ms<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.is_finite() {
        serializer.serialize_f64(*value)
    } else {
        serializer.serialize_none()
    }
}

fn fmt_ms(value: f64) -> String {
    if value.is_finite() && value > 0.0 {
        format!("{value:.2}")
    } else {
        "n/a".to_string()
    }
}

fn fastest(result: &TimingResult) -> &'static str {
    [
        ("dxf-rs", result.dxf_ms),
        ("acadrust", result.acadrust_ms),
        ("ACadSharp", result.acadsharp_ms),
        ("ezdxf", result.ezdxf_ms),
    ]
    .into_iter()
    .filter(|(_, value)| value.is_finite() && *value > 0.0)
    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    .map(|(name, _)| name)
    .unwrap_or("n/a")
}

/// Write a human-readable Markdown report and a JSON sidecar next to it.
pub fn write_report(
    report_path: &Path,
    mode: &str,
    iterations: usize,
    sections: &[ReportSection],
) -> io::Result<(std::path::PathBuf, std::path::PathBuf)> {
    if let Some(parent) = report_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut markdown = String::new();
    markdown.push_str("# acadrust-dxf-bench report\n\n");
    markdown.push_str(&format!(
        "- Mode: `{mode}`\n- Iterations: `{iterations}`\n- Generated at (Unix): `{generated_at_unix}`\n\n"
    ));

    for section in sections {
        markdown.push_str(&format!("## {}\n\n", section.name));
        markdown.push_str(
            "| Label | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | ezdxf (ms) | Fastest |\n",
        );
        markdown.push_str("|---|---:|---:|---:|---:|---|\n");
        for result in &section.results {
            markdown.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                result.label,
                fmt_ms(result.dxf_ms),
                fmt_ms(result.acadrust_ms),
                fmt_ms(result.acadsharp_ms),
                fmt_ms(result.ezdxf_ms),
                fastest(result),
            ));
        }
        if section.results.is_empty() {
            markdown.push_str("| _No measurements_ | | | | | |\n");
        }
        markdown.push('\n');
    }

    fs::write(report_path, markdown)?;

    let json_path = report_path.with_extension("json");
    let document = ReportDocument {
        generated_at_unix,
        mode,
        iterations,
        sections,
    };
    let json = serde_json::to_vec_pretty(&document)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
    fs::write(&json_path, json)?;

    Ok((report_path.to_path_buf(), json_path))
}
