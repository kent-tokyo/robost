// ---- flow / canvas helpers --------------------------------------------------

use anyhow::Result;

use crate::step_templates::STEP_TEMPLATES;

pub(crate) fn get_inner_steps(step: &serde_yml::Value) -> Vec<(String, Vec<serde_yml::Value>)> {
    let m = match step.as_mapping() {
        Some(m) => m,
        None => return vec![],
    };
    let key = m.iter().next().and_then(|(k, _)| k.as_str()).unwrap_or("");
    match key {
        "if" => {
            let mut out = vec![];
            if let Some(seq) = m.get("then").and_then(|v| v.as_sequence()) {
                out.push(("then".to_owned(), seq.clone()));
            }
            if let Some(seq) = m.get("else").and_then(|v| v.as_sequence()) {
                out.push(("else".to_owned(), seq.clone()));
            }
            out
        }
        "foreach" | "repeat" | "while" | "do_while" => {
            if let Some(seq) = m
                .get(key)
                .and_then(|v| v.as_mapping())
                .and_then(|im| im.get("do"))
                .and_then(|v| v.as_sequence())
            {
                vec![("do".to_owned(), seq.clone())]
            } else {
                vec![]
            }
        }
        "try_catch" => {
            let inner = m.get("try_catch").and_then(|v| v.as_mapping());
            let mut out = vec![];
            if let Some(im) = inner {
                if let Some(seq) = im.get("try").and_then(|v| v.as_sequence()) {
                    out.push(("try".to_owned(), seq.clone()));
                }
                if let Some(seq) = im.get("catch").and_then(|v| v.as_sequence()) {
                    out.push(("catch".to_owned(), seq.clone()));
                }
            }
            out
        }
        "group" => {
            if let Some(seq) = m
                .get("group")
                .and_then(|v| v.as_mapping())
                .and_then(|im| im.get("steps"))
                .and_then(|v| v.as_sequence())
            {
                vec![("steps".to_owned(), seq.clone())]
            } else {
                vec![]
            }
        }
        "switch" => {
            let inner = m.get("switch").and_then(|v| v.as_mapping());
            let mut out = vec![];
            if let Some(im) = inner {
                if let Some(cases) = im.get("cases").and_then(|v| v.as_sequence()) {
                    for case in cases {
                        let when_label = case
                            .as_mapping()
                            .and_then(|cm| cm.get("when"))
                            .map(|w| match w {
                                serde_yml::Value::String(s) => format!("case: {s}"),
                                serde_yml::Value::Number(n) => format!("case: {n}"),
                                serde_yml::Value::Bool(b) => format!("case: {b}"),
                                _ => "case".to_owned(),
                            })
                            .unwrap_or_else(|| "case".to_owned());
                        if let Some(steps) = case
                            .as_mapping()
                            .and_then(|cm| cm.get("do"))
                            .and_then(|v| v.as_sequence())
                        {
                            out.push((when_label, steps.clone()));
                        }
                    }
                }
                if let Some(seq) = im.get("default").and_then(|v| v.as_sequence()) {
                    out.push(("default".to_owned(), seq.clone()));
                }
            }
            out
        }
        _ => vec![],
    }
}

pub(crate) fn count_child_steps(step: &serde_yml::Value) -> usize {
    let branches = get_inner_steps(step);
    let mut total = 0;
    for (_, children) in &branches {
        total += children.len();
        for child in children {
            total += count_child_steps(child);
        }
    }
    total
}

pub(crate) fn default_canvas_cols(n: usize) -> usize {
    ((n as f32).sqrt() as usize + 1).clamp(4, 8)
}

pub(crate) fn step_display_name(key: &str) -> &str {
    STEP_TEMPLATES
        .iter()
        .find(|t| t.name == key)
        .map(|t| t.display_name)
        .unwrap_or(key)
}

pub(crate) fn get_step_key(v: &serde_yml::Value) -> &str {
    v.as_mapping()
        .and_then(|m| m.iter().next())
        .and_then(|(k, _)| k.as_str())
        .unwrap_or("?")
}

pub(crate) fn step_summary(v: &serde_yml::Value) -> String {
    let map = match v.as_mapping() {
        Some(m) => m,
        None => return "(空)".into(),
    };
    if let Some((k, val)) = map.iter().next() {
        let key = k.as_str().unwrap_or("?");
        let display = step_display_name(key);
        let val_str = match val {
            serde_yml::Value::String(s) => s.clone(),
            serde_yml::Value::Number(n) => n.to_string(),
            serde_yml::Value::Bool(b) => b.to_string(),
            serde_yml::Value::Mapping(m) => {
                // Show the first value directly — the step display name already
                // conveys the type, so repeating the sub-key name ("template",
                // "text", etc.) is redundant noise in the node label.
                if let Some((_sk, sv)) = m.iter().next() {
                    match sv {
                        serde_yml::Value::String(s) => s.clone(),
                        serde_yml::Value::Number(n) => n.to_string(),
                        serde_yml::Value::Bool(b) => b.to_string(),
                        _ => "…".into(),
                    }
                } else {
                    "{}".into()
                }
            }
            _ => "…".into(),
        };
        format!("{display}: {val_str}")
    } else {
        "(空)".into()
    }
}

pub(crate) const NODE_W: f32 = 260.0;
pub(crate) const NODE_H: f32 = 72.0;

pub(crate) fn default_canvas_pos(i: usize, cols: usize) -> egui::Pos2 {
    egui::pos2(
        (i % cols) as f32 * 340.0 + 40.0,
        (i / cols) as f32 * 132.0 + 40.0,
    )
}

pub(crate) fn step_matches(step: &serde_yml::Value, query: &str) -> bool {
    step_summary(step).to_lowercase().contains(query)
        || get_step_key(step).to_lowercase().contains(query)
        || serde_yml::to_string(step)
            .unwrap_or_default()
            .to_lowercase()
            .contains(query)
}

pub(crate) fn parse_scenario_steps(
    text: &str,
) -> Result<(String, serde_yml::Mapping, Vec<serde_yml::Value>)> {
    let doc: serde_yml::Value = serde_yml::from_str(text)?;
    let name = doc
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed")
        .to_owned();
    let vars = doc
        .get("variables")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    let steps = doc
        .get("steps")
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_default();
    Ok((name, vars, steps))
}

pub(crate) fn build_scenario_yaml(
    name: &str,
    vars: &serde_yml::Mapping,
    steps: &[serde_yml::Value],
) -> Result<String> {
    let mut map = serde_yml::Mapping::new();
    map.insert(
        serde_yml::Value::String("name".into()),
        serde_yml::Value::String(name.into()),
    );
    if !vars.is_empty() {
        map.insert(
            serde_yml::Value::String("variables".into()),
            serde_yml::Value::Mapping(vars.clone()),
        );
    }
    map.insert(
        serde_yml::Value::String("steps".into()),
        serde_yml::Value::Sequence(steps.to_vec()),
    );
    Ok(serde_yml::to_string(&serde_yml::Value::Mapping(map))?)
}
