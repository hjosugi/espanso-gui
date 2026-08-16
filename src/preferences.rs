use crate::espanso;
use crate::i18n::Language;
use crate::snippet_library::SnippetSort;
use crate::theme;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Preferences {
    pub(crate) config_root: PathBuf,
    #[serde(default)]
    pub(crate) language: Language,
    #[serde(default = "default_ui_scale")]
    pub(crate) ui_scale: f32,
    #[serde(default)]
    pub(crate) snippet_sort: SnippetSort,
    #[serde(default)]
    pub(crate) appearance: theme::Appearance,
}

pub(crate) fn default_ui_scale() -> f32 {
    1.0
}

pub(crate) fn format_ui_scale(value: f64, _decimals: std::ops::RangeInclusive<usize>) -> String {
    format!("{:.0}%", value * 100.0)
}

pub(crate) fn parse_ui_scale(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_end_matches('%')
        .trim()
        .parse::<f64>()
        .ok()
        .map(|percentage| percentage / 100.0)
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            config_root: espanso::default_config_root(),
            language: Language::default(),
            ui_scale: default_ui_scale(),
            snippet_sort: SnippetSort::default(),
            appearance: theme::Appearance::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_scale_percentage_is_round_trippable() {
        assert_eq!(
            format_ui_scale(f64::from(theme::UI_SCALE_MIN), 0..=2),
            "80%"
        );
        assert_eq!(
            format_ui_scale(f64::from(theme::UI_SCALE_MAX), 0..=2),
            "200%"
        );
        assert_eq!(parse_ui_scale("150%"), Some(1.5));
        assert_eq!(parse_ui_scale(" 125 % "), Some(1.25));
        assert_eq!(parse_ui_scale("not a percentage"), None);
    }

    #[test]
    fn legacy_preferences_receive_current_optional_defaults() {
        let preferences: Preferences = serde_json::from_str(
            r#"{"config_root":"/tmp/espanso","language":"english","ui_scale":1.0}"#,
        )
        .expect("legacy preferences");

        assert_eq!(preferences.snippet_sort, SnippetSort::FileOrder);
        assert_eq!(preferences.appearance, theme::Appearance::System);
    }
}
