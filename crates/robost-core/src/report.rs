use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum StepOutcome {
    Ok,
    Skipped,
    Failed(String),
}

impl StepOutcome {
    fn label(&self) -> &str {
        match self {
            Self::Ok => "OK",
            Self::Skipped => "SKIPPED",
            Self::Failed(_) => "FAILED",
        }
    }

    fn icon(&self) -> &str {
        match self {
            Self::Ok => "&#10003;",
            Self::Skipped => "&#8212;",
            Self::Failed(_) => "&#10007;",
        }
    }

    fn css_class(&self) -> &str {
        match self {
            Self::Ok => "ok",
            Self::Skipped => "skip",
            Self::Failed(_) => "fail",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Failed(m) => m.as_str(),
            _ => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Success,
    Failed { step_index: usize, message: String },
}

#[derive(Debug, Clone)]
pub struct StepRecord {
    pub index: usize,
    pub name: String,
    pub started_at: DateTime<Local>,
    pub elapsed_ms: u64,
    pub outcome: StepOutcome,
    pub screenshot_path: Option<PathBuf>,
    /// NCC match score for image-matching steps (None for other step types).
    pub confidence: Option<f32>,
    /// JSON snapshot of all variables at the time of failure (None on success).
    pub vars_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionReport {
    pub scenario_name: String,
    pub started_at: DateTime<Local>,
    pub finished_at: DateTime<Local>,
    pub steps: Vec<StepRecord>,
    pub outcome: Outcome,
}

impl ExecutionReport {
    pub fn write_csv(&self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        writeln!(
            f,
            "index,name,outcome,started_at,elapsed_ms,confidence,message,screenshot"
        )?;
        for s in &self.steps {
            let screenshot = s
                .screenshot_path
                .as_deref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let msg = s.outcome.message().replace('"', "\"\"");
            let conf = s
                .confidence
                .map(|c| format!("{c:.3}"))
                .unwrap_or_default();
            writeln!(
                f,
                "{},{},{},{},{},{},\"{}\",{}",
                s.index,
                s.name,
                s.outcome.label(),
                s.started_at.format("%Y-%m-%d %H:%M:%S"),
                s.elapsed_ms,
                conf,
                msg,
                screenshot,
            )?;
        }
        let total_ms = self
            .finished_at
            .signed_duration_since(self.started_at)
            .num_milliseconds();
        let result = match &self.outcome {
            Outcome::Success => "SUCCESS".to_owned(),
            Outcome::Failed { message, .. } => format!("FAILED: {message}"),
        };
        writeln!(
            f,
            ",,TOTAL,{},{},\"{}\"",
            self.started_at.format("%Y-%m-%d %H:%M:%S"),
            total_ms,
            result,
        )?;
        Ok(())
    }

    pub fn write_html(&self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let total_ms = self
            .finished_at
            .signed_duration_since(self.started_at)
            .num_milliseconds();
        let (summary_class, summary_text) = match &self.outcome {
            Outcome::Success => ("ok", "SUCCESS".to_owned()),
            Outcome::Failed {
                step_index,
                message,
            } => ("fail", format!("FAILED at step {step_index}: {message}")),
        };

        let mut rows = String::new();
        for s in &self.steps {
            let cls = s.outcome.css_class();
            let icon = s.outcome.icon();
            let msg = html_escape(s.outcome.message());
            let conf_cell = s
                .confidence
                .map(|c| format!("{c:.3}"))
                .unwrap_or_default();
            let mut detail = String::new();
            if !msg.is_empty() {
                detail.push_str(&msg);
            }
            if let Some(p) = &s.screenshot_path {
                let href = html_escape(&p.display().to_string());
                detail.push_str(&format!(" <a href=\"{href}\">screenshot</a>"));
            }
            if let Some(vars) = &s.vars_json {
                detail.push_str(&format!(
                    "<details><summary>variables</summary><pre>{}</pre></details>",
                    html_escape(vars)
                ));
            }
            rows.push_str(&format!(
                "<tr class=\"{cls}\"><td>{}</td><td>{}</td>\
                 <td class=\"outcome\">{icon} {}</td>\
                 <td>{}</td><td>{}</td><td>{conf_cell}</td><td>{detail}</td></tr>\n",
                s.index,
                html_escape(&s.name),
                s.outcome.label(),
                s.started_at.format("%H:%M:%S"),
                s.elapsed_ms,
            ));
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<title>実行レポート — {name}</title>
<style>
body {{ font-family: sans-serif; margin: 2em; }}
h1 {{ font-size: 1.4em; }}
.summary {{ padding: .5em 1em; border-radius: 4px; display: inline-block; margin-bottom: 1em; font-weight: bold; }}
.ok {{ background: #d4edda; color: #155724; }}
.fail {{ background: #f8d7da; color: #721c24; }}
.skip {{ background: #fff3cd; color: #856404; }}
table {{ border-collapse: collapse; width: 100%; }}
th {{ background: #343a40; color: #fff; padding: .5em .8em; text-align: left; }}
td {{ padding: .4em .8em; border-bottom: 1px solid #dee2e6; }}
tr.ok td {{ }}
tr.fail td {{ background: #fff5f5; }}
tr.skip td {{ color: #888; }}
.outcome {{ font-weight: bold; }}
</style>
</head>
<body>
<h1>実行レポート: {name}</h1>
<p>開始: {started} &nbsp; 終了: {finished} &nbsp; 所要時間: {total_ms} ms</p>
<div class="summary {summary_class}">{summary_text}</div>
<table>
<tr><th>#</th><th>ステップ</th><th>結果</th><th>開始時刻</th><th>経過(ms)</th><th>confidence</th><th>詳細</th></tr>
{rows}
</table>
</body>
</html>
"#,
            name = html_escape(&self.scenario_name),
            started = self.started_at.format("%Y-%m-%d %H:%M:%S"),
            finished = self.finished_at.format("%Y-%m-%d %H:%M:%S"),
        );

        let mut f = std::fs::File::create(path)?;
        f.write_all(html.as_bytes())?;
        Ok(())
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
