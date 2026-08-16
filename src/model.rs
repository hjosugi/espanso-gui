use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MatchFile {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub global_vars: Vec<Variable>,
    #[serde(default)]
    pub matches: Vec<Snippet>,
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Snippet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub form_fields: IndexMap<String, FormField>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_terms: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vars: Vec<Variable>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_word: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_word: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub propagate_case: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uppercase_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<bool>,
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variable {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub params: IndexMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inject_vars: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FormField {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiline: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim_string_values: Option<bool>,
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormFieldKind {
    Text,
    Multiline,
    Choice,
    List,
    Unknown(String),
}

impl FormFieldKind {
    pub const ALL: [Self; 4] = [Self::Text, Self::Multiline, Self::Choice, Self::List];
}

impl FormField {
    pub fn kind(&self) -> FormFieldKind {
        match self.r#type.as_deref() {
            Some("choice") => FormFieldKind::Choice,
            Some("list") => FormFieldKind::List,
            Some(kind) => FormFieldKind::Unknown(kind.to_owned()),
            None if self.multiline == Some(true) => FormFieldKind::Multiline,
            None => FormFieldKind::Text,
        }
    }

    pub fn set_kind(&mut self, kind: &FormFieldKind) {
        match kind {
            FormFieldKind::Choice => {
                self.r#type = Some("choice".into());
                self.multiline = None;
            }
            FormFieldKind::List => {
                self.r#type = Some("list".into());
                self.multiline = None;
            }
            FormFieldKind::Multiline => {
                self.r#type = None;
                self.multiline = Some(true);
                self.values.clear();
            }
            FormFieldKind::Text => {
                self.r#type = None;
                self.multiline = None;
                self.values.clear();
            }
            FormFieldKind::Unknown(kind) => {
                self.r#type = Some(kind.clone());
            }
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ConfigProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_exec: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_os: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apply_patch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inject_delay: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_delay: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_paste_delay: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paste_shortcut_event_delay: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_form_delay: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_search_delay: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paste_shortcut: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_form_width: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_form_height: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_shortcut: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preserve_clipboard: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_icon: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_notifications: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toggle_key: Option<String>,
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

impl ConfigProfile {
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml_ng::Error> {
        serde_yaml_ng::from_str(content)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml_ng::Error> {
        serde_yaml_ng::to_string(self)
    }

    pub fn has_filter(&self) -> bool {
        [
            self.filter_title.as_deref(),
            self.filter_exec.as_deref(),
            self.filter_class.as_deref(),
            self.filter_os.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| !value.trim().is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Plain,
    Markdown,
    Html,
    Image,
    Form,
}

impl ContentKind {
    pub const ALL: [Self; 5] = [
        Self::Plain,
        Self::Markdown,
        Self::Html,
        Self::Image,
        Self::Form,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Plain => "replace",
            Self::Markdown => "markdown",
            Self::Html => "html",
            Self::Image => "image_path",
            Self::Form => "form",
        }
    }
}

impl Snippet {
    pub fn new() -> Self {
        Self {
            trigger: Some(":new".into()),
            replace: Some(String::new()),
            ..Self::default()
        }
    }

    pub fn with_template(label: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            replace: Some(replacement.into()),
            ..Self::new()
        }
    }

    pub fn content_kind(&self) -> ContentKind {
        if self.markdown.is_some() {
            ContentKind::Markdown
        } else if self.html.is_some() {
            ContentKind::Html
        } else if self.image_path.is_some() {
            ContentKind::Image
        } else if self.form.is_some() {
            ContentKind::Form
        } else {
            ContentKind::Plain
        }
    }

    pub fn content(&self) -> &str {
        self.markdown
            .as_deref()
            .or(self.html.as_deref())
            .or(self.image_path.as_deref())
            .or(self.form.as_deref())
            .or(self.replace.as_deref())
            .unwrap_or("")
    }

    pub fn content_mut(&mut self) -> &mut String {
        match self.content_kind() {
            ContentKind::Plain => self.replace.get_or_insert_default(),
            ContentKind::Markdown => self.markdown.get_or_insert_default(),
            ContentKind::Html => self.html.get_or_insert_default(),
            ContentKind::Image => self.image_path.get_or_insert_default(),
            ContentKind::Form => self.form.get_or_insert_default(),
        }
    }

    pub fn set_content_kind(&mut self, kind: ContentKind) {
        let content = self.content().to_string();
        self.replace = None;
        self.markdown = None;
        self.html = None;
        self.image_path = None;
        self.form = None;
        match kind {
            ContentKind::Plain => self.replace = Some(content),
            ContentKind::Markdown => self.markdown = Some(content),
            ContentKind::Html => self.html = Some(content),
            ContentKind::Image => self.image_path = Some(content),
            ContentKind::Form => self.form = Some(content),
        }
    }

    pub fn trigger_list(&self) -> Vec<String> {
        if !self.triggers.is_empty() {
            self.triggers.clone()
        } else {
            self.trigger.iter().cloned().collect()
        }
    }

    pub fn set_trigger_list(&mut self, triggers: Vec<String>) {
        let triggers: Vec<_> = triggers
            .into_iter()
            .map(|trigger| trigger.trim().to_string())
            .filter(|trigger| !trigger.is_empty())
            .collect();
        if triggers.len() <= 1 {
            self.trigger = triggers.first().cloned();
            self.triggers.clear();
        } else {
            self.trigger = None;
            self.triggers = triggers;
        }
        self.regex = None;
    }

    pub fn set_regex_trigger_mode(&mut self, enabled: bool) {
        if enabled == self.regex.is_some() {
            return;
        }

        if enabled {
            let triggers = self.trigger_list();
            let regex = match triggers.as_slice() {
                [] => String::new(),
                [trigger] => escape_regex_literal(trigger),
                triggers => format!(
                    "(?:{})",
                    triggers
                        .iter()
                        .map(|trigger| escape_regex_literal(trigger))
                        .collect::<Vec<_>>()
                        .join("|")
                ),
            };
            self.trigger = None;
            self.triggers.clear();
            self.regex = Some(regex);
        } else {
            let regex = self.regex.take().unwrap_or_default();
            self.set_trigger_list(vec![regex]);
        }
    }

    pub fn title(&self) -> Option<String> {
        self.label
            .as_deref()
            .filter(|label| !label.trim().is_empty())
            .map(str::to_string)
            .or_else(|| self.trigger_list().first().cloned())
            .or_else(|| self.regex.clone())
    }

    pub fn searchable_text(&self) -> String {
        format!(
            "{} {} {} {}",
            self.title().unwrap_or_default(),
            self.trigger_list().join(" "),
            self.content(),
            self.search_terms.join(" ")
        )
        .to_lowercase()
    }

    pub fn insert_token(&mut self, token: &str) {
        let content = self.content_mut();
        if !content.is_empty() && !content.ends_with(char::is_whitespace) {
            content.push(' ');
        }
        content.push_str(token);
    }
}

fn escape_regex_literal(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if matches!(
                character,
                '\\' | '.' | '^' | '$' | '|' | '?' | '*' | '+' | '(' | ')' | '[' | ']' | '{' | '}'
            ) {
                [Some('\\'), Some(character)]
            } else {
                [Some(character), None]
            }
        })
        .flatten()
        .collect()
}

impl Variable {
    pub fn new(kind: &str) -> Self {
        let mut variable = Self {
            name: default_variable_name(kind).into(),
            kind: kind.into(),
            ..Self::default()
        };
        match kind {
            "date" => {
                variable.set_param("format", "%Y-%m-%d");
            }
            "echo" => variable.set_param("echo", ""),
            "random" => variable.set_string_list("choices", &[]),
            "choice" => variable.set_string_list("values", &[]),
            "shell" => {
                variable.set_param("cmd", "echo hello");
            }
            "script" => {
                variable.set_string_list("args", &["python".into(), "$CONFIG/script.py".into()]);
            }
            "form" => variable.set_param("layout", "[[name]]"),
            _ => {}
        }
        variable
    }

    pub fn token(&self) -> String {
        format!("{{{{{}}}}}", self.name)
    }

    pub fn param_str(&self, key: &str) -> String {
        self.params
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    }

    pub fn param_i64(&self, key: &str) -> i64 {
        self.params
            .get(key)
            .and_then(Value::as_i64)
            .unwrap_or_default()
    }

    pub fn param_bool(&self, key: &str, default: bool) -> bool {
        self.params
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    }

    pub fn param_strings(&self, key: &str) -> Vec<String> {
        match self.params.get(key) {
            Some(Value::Sequence(values)) => values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect(),
            Some(Value::String(values)) => values.lines().map(str::to_string).collect(),
            _ => Vec::new(),
        }
    }

    pub fn set_param(&mut self, key: &str, value: impl Into<String>) {
        self.params.insert(key.into(), Value::String(value.into()));
    }

    pub fn set_param_optional(&mut self, key: &str, value: &str) {
        if value.trim().is_empty() {
            self.params.shift_remove(key);
        } else {
            self.set_param(key, value.trim());
        }
    }

    pub fn set_i64(&mut self, key: &str, value: i64, omit_zero: bool) {
        if omit_zero && value == 0 {
            self.params.shift_remove(key);
        } else {
            self.params.insert(key.into(), Value::Number(value.into()));
        }
    }

    pub fn set_bool(&mut self, key: &str, value: bool, default: bool) {
        if value == default {
            self.params.shift_remove(key);
        } else {
            self.params.insert(key.into(), Value::Bool(value));
        }
    }

    pub fn set_string_list(&mut self, key: &str, values: &[String]) {
        let values = values
            .iter()
            .map(|value| Value::String(value.clone()))
            .collect();
        self.params.insert(key.into(), Value::Sequence(values));
    }

    pub fn form_fields(&self) -> IndexMap<String, FormField> {
        self.params
            .get("fields")
            .and_then(|value| serde_yaml_ng::from_value(value.clone()).ok())
            .unwrap_or_default()
    }

    pub fn set_form_fields(&mut self, fields: &IndexMap<String, FormField>) {
        if fields.is_empty() {
            self.params.shift_remove("fields");
        } else if let Ok(value) = serde_yaml_ng::to_value(fields) {
            self.params.insert("fields".into(), value);
        }
    }
}

fn default_variable_name(kind: &str) -> &str {
    match kind {
        "date" => "date",
        "clipboard" => "clipboard",
        "random" => "random",
        "choice" => "choice",
        "echo" => "value",
        "shell" | "script" => "output",
        "form" => "form1",
        "global" => "global_value",
        _ => "variable",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub snippet_index: Option<usize>,
    pub kind: DiagnosticKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    MissingTrigger,
    DuplicateTrigger {
        trigger: String,
        previous_snippet: usize,
    },
    EmptyContent,
    UndefinedVariable {
        reference: String,
    },
    InvalidVariableName {
        name: String,
    },
    MissingVariableKind {
        name: String,
    },
}

impl MatchFile {
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml_ng::Error> {
        serde_yaml_ng::from_str(content)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml_ng::Error> {
        serde_yaml_ng::to_string(self)
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut triggers: HashMap<String, usize> = HashMap::new();
        let global_names: HashSet<_> = self
            .global_vars
            .iter()
            .map(|var| var.name.as_str())
            .collect();

        for variable in &self.global_vars {
            validate_variable(variable, None, &mut diagnostics);
        }

        for (index, snippet) in self.matches.iter().enumerate() {
            let trigger_list = snippet.trigger_list();
            if trigger_list.is_empty() && snippet.regex.as_deref().unwrap_or_default().is_empty() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    snippet_index: Some(index),
                    kind: DiagnosticKind::MissingTrigger,
                });
            }
            for trigger in trigger_list {
                if let Some(previous) = triggers.insert(trigger.clone(), index) {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        snippet_index: Some(index),
                        kind: DiagnosticKind::DuplicateTrigger {
                            trigger,
                            previous_snippet: previous + 1,
                        },
                    });
                }
            }
            if snippet.content().is_empty() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    snippet_index: Some(index),
                    kind: DiagnosticKind::EmptyContent,
                });
            }
            let local_names: HashSet<_> =
                snippet.vars.iter().map(|var| var.name.as_str()).collect();
            for variable in &snippet.vars {
                validate_variable(variable, Some(index), &mut diagnostics);
            }
            for reference in variable_references(snippet.content()) {
                let root = reference.split('.').next().unwrap_or_default();
                if root != "cursor"
                    && !local_names.contains(root)
                    && !global_names.contains(root)
                    && !snippet.content().contains(&format!("[[{root}]]"))
                {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        snippet_index: Some(index),
                        kind: DiagnosticKind::UndefinedVariable { reference },
                    });
                }
            }
        }
        diagnostics
    }
}

fn validate_variable(
    variable: &Variable,
    snippet_index: Option<usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if variable.name.is_empty()
        || !variable
            .name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            snippet_index,
            kind: DiagnosticKind::InvalidVariableName {
                name: variable.name.clone(),
            },
        });
    }
    if variable.kind.trim().is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            snippet_index,
            kind: DiagnosticKind::MissingVariableKind {
                name: variable.name.clone(),
            },
        });
    }
}

pub fn variable_references(content: &str) -> Vec<String> {
    let mut references = Vec::new();
    let mut remaining = content;
    while let Some(start) = remaining.find("{{") {
        remaining = &remaining[start + 2..];
        let Some(end) = remaining.find("}}") else {
            break;
        };
        let name = remaining[..end].trim();
        if !name.is_empty() {
            references.push(name.to_string());
        }
        remaining = &remaining[end + 2..];
    }
    references
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn round_trips_supported_and_unknown_fields() {
        let yaml = r#"global_vars:
  - name: company
    type: echo
    params:
      echo: Acme
matches:
  - triggers: [":hi", ":hello"]
    replace: "Hello {{company}}"
    label: Greeting
    custom_option: keep-me
"#;
        let file = MatchFile::from_yaml(yaml).unwrap();
        assert_eq!(file.matches[0].trigger_list(), vec![":hi", ":hello"]);
        assert_eq!(
            file.matches[0].extra.get("custom_option"),
            Some(&Value::String("keep-me".into()))
        );
        let output = file.to_yaml().unwrap();
        let reparsed = MatchFile::from_yaml(&output).unwrap();
        assert_eq!(file, reparsed);
    }

    #[test]
    fn new_snippet_defaults_are_language_neutral() {
        let snippet = Snippet::new();
        assert_eq!(snippet.trigger.as_deref(), Some(":new"));
        assert_eq!(snippet.content(), "");
        assert_eq!(snippet.label, None);

        let templated = Snippet::with_template("New snippet", "Enter replacement text here");
        assert_eq!(templated.label.as_deref(), Some("New snippet"));
        assert_eq!(templated.content(), "Enter replacement text here");
    }

    #[test]
    fn trigger_mode_switches_persist_immediately_without_dropping_text() {
        let mut snippet = Snippet {
            triggers: vec![":one".into(), ":two.(*)".into()],
            replace: Some("value".into()),
            ..Snippet::default()
        };

        snippet.set_regex_trigger_mode(true);
        assert_eq!(snippet.regex.as_deref(), Some("(?::one|:two\\.\\(\\*\\))"));
        assert!(snippet.trigger.is_none());
        assert!(snippet.triggers.is_empty());

        let serialized = MatchFile {
            matches: vec![snippet.clone()],
            ..MatchFile::default()
        }
        .to_yaml()
        .expect("regex match file should serialize");
        let restored = MatchFile::from_yaml(&serialized).expect("regex match file should reload");
        assert_eq!(restored.matches[0].regex, snippet.regex);
        assert!(restored.matches[0].trigger.is_none());
        assert!(restored.matches[0].triggers.is_empty());

        snippet.set_regex_trigger_mode(false);
        assert_eq!(
            snippet.trigger.as_deref(),
            Some("(?::one|:two\\.\\(\\*\\))")
        );
        assert!(snippet.regex.is_none());
    }

    #[test]
    fn form_field_kinds_map_to_espanso_options_and_preserve_unknown_types() {
        let mut field = FormField::default();
        field.set_kind(&FormFieldKind::Multiline);
        assert_eq!(field.multiline, Some(true));

        field.set_kind(&FormFieldKind::Choice);
        assert_eq!(field.r#type.as_deref(), Some("choice"));
        assert_eq!(field.multiline, None);

        field.r#type = Some("future_widget".into());
        assert_eq!(field.kind(), FormFieldKind::Unknown("future_widget".into()));
        assert_eq!(field.r#type.as_deref(), Some("future_widget"));
    }

    #[test]
    fn content_kind_switch_keeps_content() {
        let mut snippet = Snippet::new();
        let original = snippet.content().to_string();
        snippet.set_content_kind(ContentKind::Markdown);
        assert_eq!(snippet.content_kind(), ContentKind::Markdown);
        assert_eq!(snippet.content(), original);
        assert!(snippet.replace.is_none());
    }

    #[test]
    fn variable_builder_serializes_espanso_shape() {
        let mut variable = Variable::new("date");
        variable.name = "tomorrow".into();
        variable.set_i64("offset", 86_400, true);
        let yaml = serde_yaml_ng::to_string(&variable).unwrap();
        assert!(yaml.contains("type: date"));
        assert!(yaml.contains("offset: 86400"));
        assert_eq!(variable.token(), "{{tomorrow}}");
    }

    #[test]
    fn diagnostics_find_invalid_and_missing_variables() {
        let mut file = MatchFile::default();
        let mut snippet = Snippet::new();
        snippet.replace = Some("Hello {{missing}}".into());
        snippet.vars.push(Variable {
            name: "bad-name".into(),
            kind: "echo".into(),
            ..Variable::default()
        });
        file.matches.push(snippet);
        let diagnostics = file.diagnostics();
        assert!(diagnostics.iter().any(|diagnostic| matches!(
            &diagnostic.kind,
            DiagnosticKind::InvalidVariableName { name } if name == "bad-name"
        )));
        assert!(diagnostics.iter().any(|diagnostic| matches!(
            &diagnostic.kind,
            DiagnosticKind::UndefinedVariable { reference } if reference == "missing"
        )));
    }

    #[test]
    fn extracts_variable_references() {
        assert_eq!(
            variable_references("{{date}} / {{ form1.name }}"),
            vec!["date", "form1.name"]
        );
    }

    #[test]
    fn config_profile_round_trips_documented_and_unknown_options() {
        let yaml = r#"filter_exec: Code|VSCodium
enable: true
backend: clipboard
key_delay: 5
max_form_width: 900
future_option: keep-me
"#;
        let profile = ConfigProfile::from_yaml(yaml).unwrap();
        assert!(profile.has_filter());
        assert_eq!(profile.backend.as_deref(), Some("clipboard"));
        assert_eq!(profile.key_delay, Some(5));
        assert_eq!(
            profile.extra.get("future_option"),
            Some(&Value::String("keep-me".into()))
        );
        assert_eq!(
            ConfigProfile::from_yaml(&profile.to_yaml().unwrap()).unwrap(),
            profile
        );
    }
}
