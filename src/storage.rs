use crate::lossless_yaml;
use crate::{
    conflict::{MergePlan, ResolutionChoice},
    model::{ConfigProfile, MatchFile, Snippet},
};
use atomic_write_file::AtomicWriteFile;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

const MAX_CONFIG_FILE_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("{0}")]
    Message(String),
    #[error("ファイル操作に失敗しました: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAMLが正しくありません: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    #[error("CSVを処理できません: {0}")]
    Csv(#[from] csv::Error),
}

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Clone)]
pub struct WorkspaceFile {
    pub relative_path: PathBuf,
    pub display_name: String,
    pub document: MatchFile,
    pub raw_yaml: String,
    pub base_yaml: String,
    pub saved_hash: String,
    pub modified_ms: u64,
    pub is_package: bool,
    pub dirty: bool,
    pub had_comments: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub relative_path: PathBuf,
    pub display_name: String,
    pub profile: ConfigProfile,
    pub raw_yaml: String,
    pub base_yaml: String,
    pub saved_hash: String,
    pub modified_ms: u64,
    pub is_default: bool,
    pub dirty: bool,
    pub had_comments: bool,
}

impl ConfigFile {
    pub fn refresh_raw_from_profile(&mut self) -> StorageResult<()> {
        self.raw_yaml = match ConfigProfile::from_yaml(&self.raw_yaml) {
            Ok(previous) => {
                lossless_yaml::patch_config_profile(&self.raw_yaml, &previous, &self.profile)?
            }
            Err(_) => self.profile.to_yaml()?,
        };
        self.dirty = true;
        Ok(())
    }

    pub fn apply_raw_yaml(&mut self) -> StorageResult<()> {
        self.profile = ConfigProfile::from_yaml(&self.raw_yaml)?;
        self.dirty = true;
        Ok(())
    }
}

impl WorkspaceFile {
    pub fn snippet_count(&self) -> usize {
        self.document.matches.len()
    }

    pub fn refresh_raw_from_document(&mut self) -> StorageResult<()> {
        self.raw_yaml = match MatchFile::from_yaml(&self.raw_yaml) {
            Ok(previous) => {
                lossless_yaml::patch_match_file(&self.raw_yaml, &previous, &self.document)?
            }
            Err(_) => self.document.to_yaml()?,
        };
        self.dirty = true;
        Ok(())
    }

    pub fn apply_raw_yaml(&mut self) -> StorageResult<()> {
        self.document = MatchFile::from_yaml(&self.raw_yaml)?;
        self.dirty = true;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SaveReceipt {
    pub hash: String,
    pub backup_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ExternalConflict {
    pub remote_yaml: String,
    pub remote_hash: String,
    pub plan: MergePlan,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub backup_path: PathBuf,
    pub timestamp: String,
}

pub fn initialize_root(root: &Path) -> StorageResult<()> {
    fs::create_dir_all(root.join("match"))?;
    fs::create_dir_all(root.join("config"))?;
    let base = root.join("match/base.yml");
    if !base.exists() {
        atomic_write(&base, b"matches: []\n")?;
    }
    Ok(())
}

pub fn load_workspace(root: &Path) -> StorageResult<Vec<WorkspaceFile>> {
    let root = canonical_root(root)?;
    let match_root = root.join("match");
    if !match_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(&match_root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if !is_yaml(path) {
            continue;
        }
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(&match_root) {
            continue;
        }
        let metadata = canonical.metadata()?;
        if metadata.len() > MAX_CONFIG_FILE_BYTES {
            continue;
        }
        let raw_yaml = fs::read_to_string(&canonical)?;
        let document = MatchFile::from_yaml(&raw_yaml)
            .map_err(|error| StorageError::Message(format!("{}: {error}", canonical.display())))?;
        let relative_path = canonical
            .strip_prefix(&root)
            .map_err(|_| StorageError::Message("設定ファイルのパスを解決できません".into()))?
            .to_path_buf();
        let normalized = relative_path.to_string_lossy().replace('\\', "/");
        files.push(WorkspaceFile {
            display_name: canonical
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or("snippets")
                .to_string(),
            relative_path,
            document,
            base_yaml: raw_yaml.clone(),
            saved_hash: hash(raw_yaml.as_bytes()),
            modified_ms: metadata
                .modified()
                .map(milliseconds_since_epoch)
                .unwrap_or_default(),
            is_package: normalized.starts_with("match/packages/"),
            dirty: false,
            had_comments: contains_yaml_comments(&raw_yaml),
            raw_yaml,
        });
    }
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

pub fn load_config_profiles(root: &Path) -> StorageResult<Vec<ConfigFile>> {
    let root = canonical_root(root)?;
    let config_root = root.join("config");
    if !config_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(&config_root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if !is_yaml(path) {
            continue;
        }
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(&config_root) {
            continue;
        }
        let metadata = canonical.metadata()?;
        if metadata.len() > MAX_CONFIG_FILE_BYTES {
            continue;
        }
        let raw_yaml = fs::read_to_string(&canonical)?;
        let profile = ConfigProfile::from_yaml(&raw_yaml)
            .map_err(|error| StorageError::Message(format!("{}: {error}", canonical.display())))?;
        let relative_path = canonical
            .strip_prefix(&root)
            .map_err(|_| StorageError::Message("設定ファイルのパスを解決できません".into()))?
            .to_path_buf();
        files.push(ConfigFile {
            display_name: canonical
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or("profile")
                .to_string(),
            is_default: relative_path == Path::new("config/default.yml")
                || relative_path == Path::new("config/default.yaml"),
            relative_path,
            profile,
            base_yaml: raw_yaml.clone(),
            saved_hash: hash(raw_yaml.as_bytes()),
            modified_ms: metadata
                .modified()
                .map(milliseconds_since_epoch)
                .unwrap_or_default(),
            dirty: false,
            had_comments: contains_yaml_comments(&raw_yaml),
            raw_yaml,
        });
    }
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

pub fn create_match_file(root: &Path, name: &str) -> StorageResult<WorkspaceFile> {
    let safe_name = normalize_file_name(name)?;
    let relative = PathBuf::from("match").join(format!("{safe_name}.yml"));
    let root = canonical_root(root)?;
    let target = checked_target(&root, &relative)?;
    if target.exists() {
        return Err(StorageError::Message(format!(
            "{} はすでに存在します",
            target.display()
        )));
    }
    let document = MatchFile::default();
    let raw_yaml = document.to_yaml()?;
    atomic_write(&target, raw_yaml.as_bytes())?;
    Ok(WorkspaceFile {
        relative_path: relative,
        display_name: safe_name,
        document,
        base_yaml: raw_yaml.clone(),
        saved_hash: hash(raw_yaml.as_bytes()),
        modified_ms: milliseconds_since_epoch(SystemTime::now()),
        is_package: false,
        dirty: false,
        had_comments: false,
        raw_yaml,
    })
}

pub fn create_config_file(root: &Path, name: &str) -> StorageResult<ConfigFile> {
    let safe_name = normalize_file_name(name)?;
    let relative = PathBuf::from("config").join(format!("{safe_name}.yml"));
    let root = canonical_root(root)?;
    let target = checked_config_target(&root, &relative)?;
    if target.exists() {
        return Err(StorageError::Message(format!(
            "{} はすでに存在します",
            target.display()
        )));
    }
    let profile = ConfigProfile::default();
    let raw_yaml = profile.to_yaml()?;
    atomic_write(&target, raw_yaml.as_bytes())?;
    Ok(ConfigFile {
        relative_path: relative,
        display_name: safe_name.clone(),
        profile,
        base_yaml: raw_yaml.clone(),
        saved_hash: hash(raw_yaml.as_bytes()),
        modified_ms: milliseconds_since_epoch(SystemTime::now()),
        is_default: safe_name == "default",
        dirty: false,
        had_comments: false,
        raw_yaml,
    })
}

pub fn save_workspace_file(root: &Path, file: &mut WorkspaceFile) -> StorageResult<SaveReceipt> {
    file.apply_raw_yaml()?;
    let root = canonical_root(root)?;
    let relative = validate_relative_path(&file.relative_path)?;
    let target = checked_target(&root, &relative)?;
    if target.exists() {
        let current = fs::read(&target)?;
        let current_hash = hash(&current);
        if current_hash != file.saved_hash {
            return Err(StorageError::Message(
                "ファイルが他のアプリで変更されました。再読み込みしてから保存してください".into(),
            ));
        }
    } else if !file.saved_hash.is_empty() {
        return Err(StorageError::Message(
            "保存先が削除されています。再読み込みしてください".into(),
        ));
    }

    let backup_path = backup_file(&root, &relative, &target)?;
    atomic_write(&target, file.raw_yaml.as_bytes())?;
    let saved_hash = hash(file.raw_yaml.as_bytes());
    file.saved_hash.clone_from(&saved_hash);
    file.base_yaml.clone_from(&file.raw_yaml);
    file.modified_ms = milliseconds_since_epoch(SystemTime::now());
    file.dirty = false;
    file.had_comments = contains_yaml_comments(&file.raw_yaml);
    Ok(SaveReceipt {
        hash: saved_hash,
        backup_path,
    })
}

pub fn save_config_file(root: &Path, file: &mut ConfigFile) -> StorageResult<SaveReceipt> {
    file.apply_raw_yaml()?;
    let root = canonical_root(root)?;
    let relative = validate_config_relative_path(&file.relative_path)?;
    let target = checked_config_target(&root, &relative)?;
    if target.exists() {
        let current = fs::read(&target)?;
        let current_hash = hash(&current);
        if current_hash != file.saved_hash {
            return Err(StorageError::Message(
                "ファイルが他のアプリで変更されました。再読み込みしてから保存してください".into(),
            ));
        }
    } else if !file.saved_hash.is_empty() {
        return Err(StorageError::Message(
            "保存先が削除されています。再読み込みしてください".into(),
        ));
    }

    let backup_path = backup_file(&root, &relative, &target)?;
    atomic_write(&target, file.raw_yaml.as_bytes())?;
    let saved_hash = hash(file.raw_yaml.as_bytes());
    file.saved_hash.clone_from(&saved_hash);
    file.base_yaml.clone_from(&file.raw_yaml);
    file.modified_ms = milliseconds_since_epoch(SystemTime::now());
    file.dirty = false;
    file.had_comments = contains_yaml_comments(&file.raw_yaml);
    Ok(SaveReceipt {
        hash: saved_hash,
        backup_path,
    })
}

pub fn move_to_recoverable_trash(root: &Path, relative_path: &Path) -> StorageResult<PathBuf> {
    let root = canonical_root(root)?;
    let relative = validate_relative_path(relative_path)?;
    let target = checked_target(&root, &relative)?;
    if !target.is_file() {
        return Err(StorageError::Message(
            "削除するファイルが見つかりません".into(),
        ));
    }
    let destination = root
        .join(".espanso-gui/trash")
        .join(timestamp())
        .join(&relative);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::rename(&target, &destination) {
        Ok(()) => {}
        Err(_) => {
            fs::copy(&target, &destination)?;
            fs::remove_file(&target)?;
        }
    }
    Ok(destination)
}

pub fn analyze_workspace_conflict(
    root: &Path,
    file: &WorkspaceFile,
) -> StorageResult<Option<ExternalConflict>> {
    analyze_external_conflict(
        root,
        &file.relative_path,
        &file.saved_hash,
        &file.base_yaml,
        &file.raw_yaml,
    )
}

pub fn analyze_config_conflict(
    root: &Path,
    file: &ConfigFile,
) -> StorageResult<Option<ExternalConflict>> {
    analyze_external_conflict(
        root,
        &file.relative_path,
        &file.saved_hash,
        &file.base_yaml,
        &file.raw_yaml,
    )
}

pub fn resolve_workspace_conflict(
    root: &Path,
    file: &mut WorkspaceFile,
    conflict: &ExternalConflict,
    choices: &[ResolutionChoice],
) -> StorageResult<SaveReceipt> {
    let value = conflict.plan.resolve(choices);
    let document: MatchFile = serde_yaml_ng::from_value(value)?;
    let remote_document = MatchFile::from_yaml(&conflict.remote_yaml)?;
    let resolved_yaml =
        lossless_yaml::patch_match_file(&conflict.remote_yaml, &remote_document, &document)?;
    let receipt = save_resolved_content(
        root,
        &file.relative_path,
        &conflict.remote_hash,
        &resolved_yaml,
    )?;
    file.document = document;
    file.raw_yaml = resolved_yaml;
    file.base_yaml.clone_from(&file.raw_yaml);
    file.saved_hash.clone_from(&receipt.hash);
    file.modified_ms = milliseconds_since_epoch(SystemTime::now());
    file.dirty = false;
    file.had_comments = contains_yaml_comments(&file.raw_yaml);
    Ok(receipt)
}

pub fn resolve_config_conflict(
    root: &Path,
    file: &mut ConfigFile,
    conflict: &ExternalConflict,
    choices: &[ResolutionChoice],
) -> StorageResult<SaveReceipt> {
    let value = conflict.plan.resolve(choices);
    let profile: ConfigProfile = serde_yaml_ng::from_value(value)?;
    let remote_profile = ConfigProfile::from_yaml(&conflict.remote_yaml)?;
    let resolved_yaml =
        lossless_yaml::patch_config_profile(&conflict.remote_yaml, &remote_profile, &profile)?;
    let receipt = save_resolved_content(
        root,
        &file.relative_path,
        &conflict.remote_hash,
        &resolved_yaml,
    )?;
    file.profile = profile;
    file.raw_yaml = resolved_yaml;
    file.base_yaml.clone_from(&file.raw_yaml);
    file.saved_hash.clone_from(&receipt.hash);
    file.modified_ms = milliseconds_since_epoch(SystemTime::now());
    file.dirty = false;
    file.had_comments = contains_yaml_comments(&file.raw_yaml);
    Ok(receipt)
}

pub fn list_history(root: &Path, relative_path: &Path) -> StorageResult<Vec<HistoryEntry>> {
    let root = canonical_root(root)?;
    let relative = validate_any_relative_path(relative_path)?;
    let backups = root.join(".espanso-gui/backups");
    if !backups.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for timestamp in fs::read_dir(backups)? {
        let timestamp = timestamp?;
        if !timestamp.file_type()?.is_dir() {
            continue;
        }
        let candidate = timestamp.path().join(&relative);
        if candidate.is_file() {
            entries.push(HistoryEntry {
                backup_path: candidate,
                timestamp: timestamp.file_name().to_string_lossy().into_owned(),
            });
        }
    }
    entries.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
    Ok(entries)
}

pub fn restore_history(
    root: &Path,
    relative_path: &Path,
    history_path: &Path,
) -> StorageResult<SaveReceipt> {
    let root = canonical_root(root)?;
    let relative = validate_any_relative_path(relative_path)?;
    let backup_root = root.join(".espanso-gui/backups");
    let history = history_path.canonicalize()?;
    if !history.starts_with(&backup_root) || !history.is_file() {
        return Err(StorageError::Message(
            "アプリのバックアップ以外からは復元できません".into(),
        ));
    }
    let target = checked_validated_target(&root, &relative)?;
    let restored = fs::read(&history)?;
    if restored.len() as u64 > MAX_CONFIG_FILE_BYTES {
        return Err(StorageError::Message(
            "復元する設定ファイルが大きすぎます".into(),
        ));
    }
    if relative.starts_with("match") {
        MatchFile::from_yaml(
            std::str::from_utf8(&restored)
                .map_err(|_| StorageError::Message("バックアップがUTF-8ではありません".into()))?,
        )?;
    } else {
        ConfigProfile::from_yaml(
            std::str::from_utf8(&restored)
                .map_err(|_| StorageError::Message("バックアップがUTF-8ではありません".into()))?,
        )?;
    }
    let backup_path = backup_file(&root, &relative, &target)?;
    atomic_write(&target, &restored)?;
    Ok(SaveReceipt {
        hash: hash(&restored),
        backup_path,
    })
}

fn analyze_external_conflict(
    root: &Path,
    relative_path: &Path,
    saved_hash: &str,
    base_yaml: &str,
    local_yaml: &str,
) -> StorageResult<Option<ExternalConflict>> {
    let root = canonical_root(root)?;
    let relative = validate_any_relative_path(relative_path)?;
    let target = checked_validated_target(&root, &relative)?;
    if !target.is_file() {
        return Err(StorageError::Message(
            "保存先が削除されています。再読み込みしてください".into(),
        ));
    }
    let remote_yaml = fs::read_to_string(target)?;
    let remote_hash = hash(remote_yaml.as_bytes());
    if remote_hash == saved_hash {
        return Ok(None);
    }
    let base = serde_yaml_ng::from_str(base_yaml)?;
    let local = serde_yaml_ng::from_str(local_yaml)?;
    let disk = serde_yaml_ng::from_str(&remote_yaml)?;
    Ok(Some(ExternalConflict {
        remote_yaml,
        remote_hash,
        plan: MergePlan::new(&base, &local, &disk),
    }))
}

fn save_resolved_content(
    root: &Path,
    relative_path: &Path,
    expected_remote_hash: &str,
    content: &str,
) -> StorageResult<SaveReceipt> {
    let root = canonical_root(root)?;
    let relative = validate_any_relative_path(relative_path)?;
    let target = checked_validated_target(&root, &relative)?;
    let current = fs::read(&target)?;
    if hash(&current) != expected_remote_hash {
        return Err(StorageError::Message(
            "競合確認後にファイルが再度変更されました。もう一度比較してください".into(),
        ));
    }
    let backup_path = backup_file(&root, &relative, &target)?;
    atomic_write(&target, content.as_bytes())?;
    Ok(SaveReceipt {
        hash: hash(content.as_bytes()),
        backup_path,
    })
}

pub fn create_backup_snapshot(root: &Path, destination_root: &Path) -> StorageResult<PathBuf> {
    let root = canonical_root(root)?;
    fs::create_dir_all(destination_root)?;
    let destination = destination_root.join(format!("espanso-backup-{}", timestamp()));
    for entry in WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let source = entry.path();
        let relative = source
            .strip_prefix(&root)
            .map_err(|_| StorageError::Message("バックアップパスを解決できません".into()))?;
        if relative.starts_with(".espanso-gui") {
            continue;
        }
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, target)?;
    }
    Ok(destination)
}

pub fn export_csv(file: &WorkspaceFile, destination: &Path) -> StorageResult<()> {
    let mut writer = csv::Writer::from_path(destination)?;
    writer.write_record(["trigger", "replacement", "label", "type"])?;
    for snippet in &file.document.matches {
        writer.write_record([
            snippet.trigger_list().join("|"),
            snippet.content().to_string(),
            snippet.label.clone().unwrap_or_default(),
            snippet.content_kind().key().to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

pub fn import_csv(file: &mut WorkspaceFile, source: &Path) -> StorageResult<usize> {
    let mut reader = csv::Reader::from_path(source)?;
    let mut imported = 0;
    for record in reader.records() {
        let record = record?;
        let trigger = record.get(0).unwrap_or_default();
        if trigger.trim().is_empty() {
            continue;
        }
        let mut snippet = Snippet::new();
        snippet.set_trigger_list(trigger.split('|').map(str::to_string).collect());
        snippet.replace = Some(record.get(1).unwrap_or_default().to_string());
        snippet.label = record
            .get(2)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        file.document.matches.push(snippet);
        imported += 1;
    }
    file.refresh_raw_from_document()?;
    Ok(imported)
}

fn normalize_file_name(name: &str) -> StorageResult<String> {
    let trimmed = name.trim();
    let value = trimmed
        .strip_suffix(".yaml")
        .or_else(|| trimmed.strip_suffix(".yml"))
        .unwrap_or(trimmed);
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(StorageError::Message(
            "ファイル名には英数字、ハイフン、アンダースコアを使用してください".into(),
        ));
    }
    Ok(value.into())
}

fn validate_relative_path(path: &Path) -> StorageResult<PathBuf> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(StorageError::Message("相対パスを指定してください".into()));
    }
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(StorageError::Message(
            "設定フォルダ外のパスは使用できません".into(),
        ));
    }
    let mut components = path.components().filter_map(|component| match component {
        Component::Normal(value) => Some(value),
        _ => None,
    });
    if components.next() != Some(OsStr::new("match")) {
        return Err(StorageError::Message(
            "スニペットはmatchフォルダ内に保存してください".into(),
        ));
    }
    if !is_yaml(path) {
        return Err(StorageError::Message(
            "拡張子は.ymlまたは.yamlにしてください".into(),
        ));
    }
    Ok(path.to_path_buf())
}

fn validate_config_relative_path(path: &Path) -> StorageResult<PathBuf> {
    validate_relative_yaml_path(
        path,
        "config",
        "設定プロファイルはconfigフォルダ内に保存してください",
    )
}

fn validate_any_relative_path(path: &Path) -> StorageResult<PathBuf> {
    match path.components().next() {
        Some(Component::Normal(value)) if value == OsStr::new("match") => {
            validate_relative_path(path)
        }
        Some(Component::Normal(value)) if value == OsStr::new("config") => {
            validate_config_relative_path(path)
        }
        _ => Err(StorageError::Message(
            "matchまたはconfigフォルダ内の設定だけを操作できます".into(),
        )),
    }
}

fn validate_relative_yaml_path(
    path: &Path,
    required_root: &str,
    wrong_root_message: &str,
) -> StorageResult<PathBuf> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(StorageError::Message("相対パスを指定してください".into()));
    }
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(StorageError::Message(
            "設定フォルダ外のパスは使用できません".into(),
        ));
    }
    let mut components = path.components().filter_map(|component| match component {
        Component::Normal(value) => Some(value),
        _ => None,
    });
    if components.next() != Some(OsStr::new(required_root)) {
        return Err(StorageError::Message(wrong_root_message.into()));
    }
    if !is_yaml(path) {
        return Err(StorageError::Message(
            "拡張子は.ymlまたは.yamlにしてください".into(),
        ));
    }
    Ok(path.to_path_buf())
}

fn checked_target(root: &Path, relative: &Path) -> StorageResult<PathBuf> {
    let relative = validate_relative_path(relative)?;
    let target = root.join(relative);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
        let parent = parent.canonicalize()?;
        if !parent.starts_with(root) {
            return Err(StorageError::Message(
                "設定フォルダ外には保存できません".into(),
            ));
        }
    }
    Ok(target)
}

fn checked_config_target(root: &Path, relative: &Path) -> StorageResult<PathBuf> {
    let relative = validate_config_relative_path(relative)?;
    checked_validated_target(root, &relative)
}

fn checked_validated_target(root: &Path, relative: &Path) -> StorageResult<PathBuf> {
    let target = root.join(relative);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
        let parent = parent.canonicalize()?;
        if !parent.starts_with(root) {
            return Err(StorageError::Message(
                "設定フォルダ外には保存できません".into(),
            ));
        }
    }
    Ok(target)
}

fn canonical_root(root: &Path) -> StorageResult<PathBuf> {
    if !root.exists() {
        return Err(StorageError::Message(format!(
            "Espanso設定フォルダが見つかりません: {}",
            root.display()
        )));
    }
    Ok(root.canonicalize()?)
}

fn backup_file(root: &Path, relative: &Path, source: &Path) -> StorageResult<Option<PathBuf>> {
    if !source.exists() {
        return Ok(None);
    }
    let backup_root = root.join(".espanso-gui/backups");
    let stamp = timestamp();
    let mut destination = backup_root.join(&stamp).join(relative);
    for sequence in 1..=1000 {
        if !destination.exists() {
            break;
        }
        destination = backup_root
            .join(format!("{stamp}-{sequence:03}"))
            .join(relative);
    }
    if destination.exists() {
        return Err(StorageError::Message(
            "backup履歴の一意な保存先を作成できません".into(),
        ));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, &destination)?;
    Ok(Some(destination))
}

fn atomic_write(path: &Path, content: &[u8]) -> StorageResult<()> {
    let mut writer = AtomicWriteFile::open(path)?;
    writer.write_all(content)?;
    writer.commit()?;
    Ok(())
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("yml" | "yaml")
    )
}

fn hash(content: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let digest = Sha256::digest(content);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

fn milliseconds_since_epoch(value: SystemTime) -> u64 {
    value
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn timestamp() -> String {
    Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string()
}

fn contains_yaml_comments(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with('#') || line.contains(" #")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn hashes_content_as_stable_lowercase_sha256() {
        assert_eq!(
            hash(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn initializes_and_loads_workspace() {
        let temp = tempfile::tempdir().unwrap();
        initialize_root(temp.path()).unwrap();
        let files = load_workspace(temp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, PathBuf::from("match/base.yml"));
    }

    #[test]
    fn save_detects_external_changes_and_makes_backup() {
        let temp = tempfile::tempdir().unwrap();
        initialize_root(temp.path()).unwrap();
        let mut file = load_workspace(temp.path()).unwrap().remove(0);
        file.document.matches.push(Snippet::new());
        file.refresh_raw_from_document().unwrap();
        let receipt = save_workspace_file(temp.path(), &mut file).unwrap();
        assert!(receipt.backup_path.unwrap().is_file());

        fs::write(
            temp.path().join("match/base.yml"),
            "matches: []\n# external\n",
        )
        .unwrap();
        assert!(save_workspace_file(temp.path(), &mut file).is_err());
    }

    #[test]
    fn delete_is_recoverable() {
        let temp = tempfile::tempdir().unwrap();
        initialize_root(temp.path()).unwrap();
        let destination =
            move_to_recoverable_trash(temp.path(), Path::new("match/base.yml")).unwrap();
        assert!(destination.is_file());
        assert!(!temp.path().join("match/base.yml").exists());
    }

    #[test]
    fn csv_round_trip_imports_basic_snippets() {
        let temp = tempfile::tempdir().unwrap();
        let mut file = WorkspaceFile {
            relative_path: "match/test.yml".into(),
            display_name: "test".into(),
            document: MatchFile {
                matches: vec![Snippet::new()],
                ..MatchFile::default()
            },
            raw_yaml: String::new(),
            base_yaml: String::new(),
            saved_hash: String::new(),
            modified_ms: 0,
            is_package: false,
            dirty: false,
            had_comments: false,
        };
        let csv_path = temp.path().join("snippets.csv");
        export_csv(&file, &csv_path).unwrap();
        file.document.matches.clear();
        assert_eq!(import_csv(&mut file, &csv_path).unwrap(), 1);
        assert_eq!(file.document.matches[0].trigger_list(), vec![":new"]);
    }

    #[test]
    fn rejects_unsafe_paths_and_file_names() {
        assert!(validate_relative_path(Path::new("../secret.yml")).is_err());
        assert!(validate_relative_path(Path::new("config/default.yml")).is_err());
        assert!(validate_config_relative_path(Path::new("../secret.yml")).is_err());
        assert!(validate_config_relative_path(Path::new("match/base.yml")).is_err());
        assert!(normalize_file_name("../../oops").is_err());
    }

    #[test]
    fn config_profile_save_is_validated_atomic_and_backed_up() {
        let temp = tempfile::tempdir().unwrap();
        initialize_root(temp.path()).unwrap();
        let mut file = create_config_file(temp.path(), "telegram").unwrap();
        file.profile.filter_exec = Some("Telegram".into());
        file.profile.enable = Some(false);
        file.refresh_raw_from_profile().unwrap();

        let receipt = save_config_file(temp.path(), &mut file).unwrap();
        assert!(receipt.backup_path.unwrap().is_file());
        let saved = fs::read_to_string(temp.path().join("config/telegram.yml")).unwrap();
        assert!(saved.contains("filter_exec: Telegram"));
        assert!(saved.contains("enable: false"));
    }

    #[test]
    fn three_way_merge_and_history_preserve_both_independent_changes() {
        let temp = tempfile::tempdir().unwrap();
        initialize_root(temp.path()).unwrap();
        let mut setup = load_workspace(temp.path()).unwrap().remove(0);
        setup.document.matches.push(Snippet::new());
        setup.refresh_raw_from_document().unwrap();
        save_workspace_file(temp.path(), &mut setup).unwrap();

        let mut local = load_workspace(temp.path()).unwrap().remove(0);
        local.document.matches[0].set_trigger_list(vec![":local".into()]);
        local.refresh_raw_from_document().unwrap();

        let mut disk_document = MatchFile::from_yaml(&local.base_yaml).unwrap();
        disk_document.matches[0].replace = Some("disk value".into());
        fs::write(
            temp.path().join("match/base.yml"),
            disk_document.to_yaml().unwrap(),
        )
        .unwrap();

        let conflict = analyze_workspace_conflict(temp.path(), &local)
            .unwrap()
            .unwrap();
        assert!(conflict.plan.conflicts.is_empty());
        resolve_workspace_conflict(temp.path(), &mut local, &conflict, &[]).unwrap();
        assert_eq!(local.document.matches[0].trigger_list(), vec![":local"]);
        assert_eq!(
            local.document.matches[0].replace.as_deref(),
            Some("disk value")
        );

        let history = list_history(temp.path(), Path::new("match/base.yml")).unwrap();
        assert!(history.len() >= 2);
        restore_history(
            temp.path(),
            Path::new("match/base.yml"),
            &history[0].backup_path,
        )
        .unwrap();
        MatchFile::from_yaml(&fs::read_to_string(temp.path().join("match/base.yml")).unwrap())
            .unwrap();
    }
}
