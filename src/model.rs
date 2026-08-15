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

    pub fn label(self) -> &'static str {
        match self {
            Self::Plain => "テキスト",
            Self::Markdown => "Markdown",
            Self::Html => "HTML",
            Self::Image => "画像",
            Self::Form => "フォーム",
        }
    }

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
            replace: Some("ここに展開するテキストを入力".into()),
            label: Some("新しいスニペット".into()),
            ..Self::default()
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

    pub fn title(&self) -> String {
        self.label
            .as_deref()
            .filter(|label| !label.trim().is_empty())
            .map(str::to_string)
            .or_else(|| self.trigger_list().first().cloned())
            .or_else(|| self.regex.clone())
            .unwrap_or_else(|| "名称未設定".into())
    }

    pub fn searchable_text(&self) -> String {
        format!(
            "{} {} {} {}",
            self.title(),
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
            "echo" => {
                variable.set_param("echo", "値");
            }
            "random" => {
                variable.set_string_list("choices", &["候補1".into(), "候補2".into()]);
            }
            "choice" => {
                variable.set_string_list("values", &["候補1".into(), "候補2".into()]);
            }
            "shell" => {
                variable.set_param("cmd", "echo hello");
            }
            "script" => {
                variable.set_string_list("args", &["python".into(), "$CONFIG/script.py".into()]);
            }
            "form" => {
                variable.set_param("layout", "名前: [[name]]");
            }
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
    pub message: String,
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
                    message: "トリガーまたは正規表現が必要です".into(),
                });
            }
            for trigger in trigger_list {
                if let Some(previous) = triggers.insert(trigger.clone(), index) {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        snippet_index: Some(index),
                        message: format!(
                            "トリガー「{trigger}」は{}番目のスニペットでも使われています",
                            previous + 1
                        ),
                    });
                }
            }
            if snippet.content().is_empty() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    snippet_index: Some(index),
                    message: "展開内容が空です".into(),
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
                        message: format!("変数「{reference}」が定義されていません"),
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
            message: format!(
                "変数名「{}」には英数字とアンダースコアだけを使用できます",
                variable.name
            ),
        });
    }
    if variable.kind.trim().is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            snippet_index,
            message: format!("変数「{}」の種類が未設定です", variable.name),
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
        let messages: Vec<_> = file
            .diagnostics()
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect();
        assert!(messages.iter().any(|message| message.contains("bad-name")));
        assert!(messages.iter().any(|message| message.contains("missing")));
    }

    #[test]
    fn extracts_variable_references() {
        assert_eq!(
            variable_references("{{date}} / {{ form1.name }}"),
            vec!["date", "form1.name"]
        );
    }
}
