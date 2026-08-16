use serde_yaml_ng::{Mapping, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionChoice {
    Local,
    Disk,
}

#[derive(Debug, Clone)]
pub struct FieldConflict {
    path: Vec<PathSegment>,
    pub label: String,
    pub base: Option<Value>,
    pub local: Option<Value>,
    pub disk: Option<Value>,
}

impl FieldConflict {
    pub fn base_summary(&self, missing: &str, unavailable: &str) -> String {
        value_summary(self.base.as_ref(), missing, unavailable)
    }

    pub fn local_summary(&self, missing: &str, unavailable: &str) -> String {
        value_summary(self.local.as_ref(), missing, unavailable)
    }

    pub fn disk_summary(&self, missing: &str, unavailable: &str) -> String {
        value_summary(self.disk.as_ref(), missing, unavailable)
    }
}

#[derive(Debug, Clone)]
pub struct MergePlan {
    merged_with_local: Option<Value>,
    pub conflicts: Vec<FieldConflict>,
}

impl MergePlan {
    pub fn new(base: &Value, local: &Value, disk: &Value) -> Self {
        let mut conflicts = Vec::new();
        let merged_with_local = merge_node(
            Some(base),
            Some(local),
            Some(disk),
            &mut Vec::new(),
            &mut conflicts,
        );
        Self {
            merged_with_local,
            conflicts,
        }
    }

    pub fn resolve(&self, choices: &[ResolutionChoice]) -> Value {
        let mut merged = self.merged_with_local.clone().unwrap_or(Value::Null);
        for (conflict, choice) in self.conflicts.iter().zip(choices) {
            if *choice == ResolutionChoice::Disk {
                set_path(&mut merged, &conflict.path, conflict.disk.clone());
            }
        }
        merged
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathSegment {
    Key(Value),
    Index(usize),
}

fn merge_node(
    base: Option<&Value>,
    local: Option<&Value>,
    disk: Option<&Value>,
    path: &mut Vec<PathSegment>,
    conflicts: &mut Vec<FieldConflict>,
) -> Option<Value> {
    if local == disk {
        return local.cloned();
    }
    if local == base {
        return disk.cloned();
    }
    if disk == base {
        return local.cloned();
    }

    match (base, local, disk) {
        (Some(Value::Mapping(base)), Some(Value::Mapping(local)), Some(Value::Mapping(disk))) => {
            let mut merged = Mapping::new();
            for key in ordered_keys(base, local, disk) {
                path.push(PathSegment::Key(key.clone()));
                if let Some(value) = merge_node(
                    base.get(&key),
                    local.get(&key),
                    disk.get(&key),
                    path,
                    conflicts,
                ) {
                    merged.insert(key, value);
                }
                path.pop();
            }
            Some(Value::Mapping(merged))
        }
        (
            Some(Value::Sequence(base)),
            Some(Value::Sequence(local)),
            Some(Value::Sequence(disk)),
        ) if base.len() == local.len() && base.len() == disk.len() => Some(Value::Sequence(
            (0..base.len())
                .filter_map(|index| {
                    path.push(PathSegment::Index(index));
                    let value = merge_node(
                        base.get(index),
                        local.get(index),
                        disk.get(index),
                        path,
                        conflicts,
                    );
                    path.pop();
                    value
                })
                .collect(),
        )),
        _ => {
            conflicts.push(FieldConflict {
                path: path.clone(),
                label: path_label(path),
                base: base.cloned(),
                local: local.cloned(),
                disk: disk.cloned(),
            });
            local.cloned()
        }
    }
}

fn ordered_keys(base: &Mapping, local: &Mapping, disk: &Mapping) -> Vec<Value> {
    let mut keys = Vec::new();
    for mapping in [base, local, disk] {
        for key in mapping.keys() {
            if !keys.contains(key) {
                keys.push(key.clone());
            }
        }
    }
    keys
}

fn set_path(root: &mut Value, path: &[PathSegment], replacement: Option<Value>) {
    if path.is_empty() {
        *root = replacement.unwrap_or(Value::Null);
        return;
    }
    match (&path[0], root) {
        (PathSegment::Key(key), Value::Mapping(mapping)) if path.len() == 1 => {
            if let Some(value) = replacement {
                mapping.insert(key.clone(), value);
            } else {
                mapping.remove(key);
            }
        }
        (PathSegment::Key(key), Value::Mapping(mapping)) => {
            if let Some(value) = mapping.get_mut(key) {
                set_path(value, &path[1..], replacement);
            }
        }
        (PathSegment::Index(index), Value::Sequence(sequence)) if path.len() == 1 => {
            if let Some(value) = replacement {
                if let Some(slot) = sequence.get_mut(*index) {
                    *slot = value;
                }
            } else if *index < sequence.len() {
                sequence.remove(*index);
            }
        }
        (PathSegment::Index(index), Value::Sequence(sequence)) => {
            if let Some(value) = sequence.get_mut(*index) {
                set_path(value, &path[1..], replacement);
            }
        }
        _ => {}
    }
}

fn path_label(path: &[PathSegment]) -> String {
    if path.is_empty() {
        return "YAML".into();
    }
    let mut label = String::new();
    for segment in path {
        match segment {
            PathSegment::Key(Value::String(key)) => {
                if !label.is_empty() {
                    label.push('.');
                }
                label.push_str(key);
            }
            PathSegment::Key(key) => {
                if !label.is_empty() {
                    label.push('.');
                }
                label.push_str(&compact_yaml(key, "?"));
            }
            PathSegment::Index(index) => label.push_str(&format!("[{index}]")),
        }
    }
    label
}

fn value_summary(value: Option<&Value>, missing: &str, unavailable: &str) -> String {
    value
        .map(|value| compact_yaml(value, unavailable))
        .unwrap_or_else(|| missing.into())
}

fn compact_yaml(value: &Value, unavailable: &str) -> String {
    let text = serde_yaml_ng::to_string(value)
        .unwrap_or_else(|_| unavailable.into())
        .trim()
        .replace('\n', " ");
    let mut characters = text.chars();
    let summary: String = characters.by_ref().take(160).collect();
    if characters.next().is_some() {
        format!("{summary}…")
    } else {
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaml(source: &str) -> Value {
        serde_yaml_ng::from_str(source).unwrap()
    }

    #[test]
    fn merges_independent_field_changes_without_conflicts() {
        let base = yaml("matches:\n  - trigger: :one\n    replace: base\n");
        let local = yaml("matches:\n  - trigger: :local\n    replace: base\n");
        let disk = yaml("matches:\n  - trigger: :one\n    replace: disk\n");
        let plan = MergePlan::new(&base, &local, &disk);
        assert!(plan.conflicts.is_empty());
        assert_eq!(
            plan.resolve(&[]),
            yaml("matches:\n  - trigger: :local\n    replace: disk\n")
        );
    }

    #[test]
    fn reports_same_field_edits_and_applies_disk_choice() {
        let base = yaml("matches:\n  - trigger: :one\n    replace: base\n");
        let local = yaml("matches:\n  - trigger: :one\n    replace: local\n");
        let disk = yaml("matches:\n  - trigger: :one\n    replace: disk\n");
        let plan = MergePlan::new(&base, &local, &disk);
        assert_eq!(plan.conflicts.len(), 1);
        assert_eq!(plan.conflicts[0].label, "matches[0].replace");
        assert_eq!(
            plan.resolve(&[ResolutionChoice::Disk]),
            yaml("matches:\n  - trigger: :one\n    replace: disk\n")
        );
    }

    #[test]
    fn reports_sequence_shape_changes_as_one_field() {
        let base = yaml("matches: []\n");
        let local = yaml("matches:\n  - trigger: :local\n    replace: local\n");
        let disk = yaml("matches:\n  - trigger: :disk\n    replace: disk\n");
        let plan = MergePlan::new(&base, &local, &disk);
        assert_eq!(plan.conflicts.len(), 1);
        assert_eq!(plan.conflicts[0].label, "matches");
    }
}
