// ---- flow / canvas helpers --------------------------------------------------

use anyhow::Result;
use eframe::egui;
use std::collections::HashSet;

use crate::step_templates::STEP_TEMPLATES;
use crate::types::{category_color, step_key_category, FlowNode};

pub(crate) fn get_inner_steps(step: &serde_yml::Value) -> Vec<(&'static str, Vec<serde_yml::Value>)> {
    let m = match step.as_mapping() {
        Some(m) => m,
        None => return vec![],
    };
    let key = m.iter().next().and_then(|(k, _)| k.as_str()).unwrap_or("");
    match key {
        "if" => {
            let mut out = vec![];
            if let Some(seq) = m.get("then").and_then(|v| v.as_sequence()) {
                out.push(("then", seq.clone()));
            }
            if let Some(seq) = m.get("else").and_then(|v| v.as_sequence()) {
                out.push(("else", seq.clone()));
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
                vec![("do", seq.clone())]
            } else {
                vec![]
            }
        }
        "try_catch" => {
            let inner = m.get("try_catch").and_then(|v| v.as_mapping());
            let mut out = vec![];
            if let Some(im) = inner {
                if let Some(seq) = im.get("try").and_then(|v| v.as_sequence()) {
                    out.push(("try", seq.clone()));
                }
                if let Some(seq) = im.get("catch").and_then(|v| v.as_sequence()) {
                    out.push(("catch", seq.clone()));
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
                vec![("steps", seq.clone())]
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
                        if let Some(steps) = case
                            .as_mapping()
                            .and_then(|cm| cm.get("do"))
                            .and_then(|v| v.as_sequence())
                        {
                            out.push(("case", steps.clone()));
                        }
                    }
                }
                if let Some(seq) = im.get("default").and_then(|v| v.as_sequence()) {
                    out.push(("default", seq.clone()));
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

pub(crate) fn collect_nodes(
    steps: &[serde_yml::Value],
    depth: usize,
    expanded: &HashSet<usize>,
    nodes: &mut Vec<FlowNode>,
) {
    for (i, step) in steps.iter().enumerate() {
        let key = get_step_key(step);
        let color = category_color(step_key_category(key));
        let summary = step_summary(step);
        let is_compound = matches!(
            key,
            "if" | "foreach" | "repeat" | "while" | "do_while" | "try_catch" | "group" | "switch"
        );
        let step_idx = i;
        let (expand_key, is_expanded) = if depth == 0 && is_compound {
            (Some(i), expanded.contains(&i))
        } else {
            (None, false)
        };
        let label = if depth == 0 {
            format!("{i}  {summary}")
        } else {
            summary
        };

        nodes.push(FlowNode {
            step_idx,
            depth,
            label,
            color,
            expand_key,
            is_expanded,
            is_header: false,
        });

        if depth == 0 && is_compound && is_expanded {
            for (branch_name, branch_steps) in get_inner_steps(step) {
                nodes.push(FlowNode {
                    step_idx: i,
                    depth: depth + 1,
                    label: format!("─ {branch_name}:"),
                    color: egui::Color32::from_gray(100),
                    expand_key: None,
                    is_expanded: false,
                    is_header: true,
                });
                for inner in &branch_steps {
                    let inner_color = category_color(step_key_category(get_step_key(inner)));
                    nodes.push(FlowNode {
                        step_idx: i,
                        depth: depth + 2,
                        label: step_summary(inner),
                        color: inner_color,
                        expand_key: None,
                        is_expanded: false,
                        is_header: false,
                    });
                }
            }
        }
    }
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
                if let Some((sk, sv)) = m.iter().next() {
                    format!(
                        "{}: {}",
                        sk.as_str().unwrap_or("?"),
                        sv.as_str().unwrap_or("…")
                    )
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

pub(crate) fn parse_scenario_steps(text: &str) -> Result<(String, serde_yml::Mapping, Vec<serde_yml::Value>)> {
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
