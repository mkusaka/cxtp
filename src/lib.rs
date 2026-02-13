use anyhow::Context;
use anyhow::Result;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use toml_edit::value;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Table;

const CONFIG_TOML_FILE: &str = "config.toml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustLevel {
    Trusted,
    Untrusted,
}

impl TrustLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
        }
    }
}

impl Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct SetTrustResult {
    pub project_path: PathBuf,
    pub config_path: PathBuf,
    pub trust_level: TrustLevel,
    pub changed: bool,
}

pub fn set_project_trust(
    project_dir: &Path,
    codex_home_override: Option<&Path>,
    trust_level: TrustLevel,
) -> Result<SetTrustResult> {
    let project_path = canonicalize_project_dir(project_dir)?;
    let codex_home = resolve_codex_home(codex_home_override)?;
    let config_path = codex_home.join(CONFIG_TOML_FILE);

    let existing = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read config file: {}", config_path.display()));
        }
    };

    let updated = upsert_project_trust(&existing, &project_path, trust_level)?;
    let changed = existing != updated;
    if changed {
        fs::create_dir_all(&codex_home)
            .with_context(|| format!("failed to create {}", codex_home.display()))?;
        fs::write(&config_path, updated)
            .with_context(|| format!("failed to write {}", config_path.display()))?;
    }

    Ok(SetTrustResult {
        project_path,
        config_path,
        trust_level,
        changed,
    })
}

pub fn resolve_codex_home(codex_home_override: Option<&Path>) -> Result<PathBuf> {
    if let Some(codex_home) = codex_home_override {
        return Ok(codex_home.to_path_buf());
    }

    if let Some(codex_home_env) = std::env::var_os("CODEX_HOME") {
        let env_path = PathBuf::from(codex_home_env);
        let canonical = fs::canonicalize(&env_path).with_context(|| {
            format!(
                "CODEX_HOME must point to an existing directory: {}",
                env_path.display()
            )
        })?;
        if !canonical.is_dir() {
            anyhow::bail!(
                "CODEX_HOME must point to a directory: {}",
                canonical.display()
            );
        }
        return Ok(canonical);
    }

    let home = dirs::home_dir().context("failed to resolve home directory")?;
    Ok(home.join(".codex"))
}

pub fn canonicalize_project_dir(project_dir: &Path) -> Result<PathBuf> {
    fs::canonicalize(project_dir).with_context(|| {
        format!(
            "failed to canonicalize project directory: {}",
            project_dir.display()
        )
    })
}

pub fn upsert_project_trust(
    config_contents: &str,
    project_path: &Path,
    trust_level: TrustLevel,
) -> Result<String> {
    if !project_path.is_absolute() {
        anyhow::bail!(
            "project path must be absolute (canonicalize before call): {}",
            project_path.display()
        );
    }

    let mut doc = if config_contents.trim().is_empty() {
        DocumentMut::new()
    } else {
        config_contents
            .parse::<DocumentMut>()
            .context("config.toml is not valid TOML")?
    };

    set_project_trust_level_inner(&mut doc, project_path, trust_level)
        .context("failed to update projects trust configuration")?;
    Ok(doc.to_string())
}

fn set_project_trust_level_inner(
    doc: &mut DocumentMut,
    project_path: &Path,
    trust_level: TrustLevel,
) -> Result<()> {
    let project_key = project_path.to_string_lossy().to_string();

    {
        let root = doc.as_table_mut();
        let existing_projects = root.get("projects").cloned();
        if existing_projects
            .as_ref()
            .is_none_or(|item| !item.is_table())
        {
            let mut projects_table = Table::new();
            projects_table.set_implicit(true);

            if let Some(inline_table) = existing_projects.as_ref().and_then(Item::as_inline_table) {
                for (key, value) in inline_table.iter() {
                    if let Some(inner_table) = value.as_inline_table() {
                        let new_table = inner_table.clone().into_table();
                        projects_table.insert(key, Item::Table(new_table));
                    }
                }
            }

            root.insert("projects", Item::Table(projects_table));
        }
    }

    let Some(projects_table) = doc["projects"].as_table_mut() else {
        anyhow::bail!("projects table missing after initialization");
    };

    let needs_project_table = !projects_table.contains_key(project_key.as_str())
        || projects_table
            .get(project_key.as_str())
            .and_then(Item::as_table)
            .is_none();
    if needs_project_table {
        projects_table.insert(project_key.as_str(), toml_edit::table());
    }

    let Some(project_table) = projects_table
        .get_mut(project_key.as_str())
        .and_then(Item::as_table_mut)
    else {
        anyhow::bail!("project table missing for {}", project_key);
    };
    project_table.set_implicit(false);
    project_table["trust_level"] = value(trust_level.to_string());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_creates_project_table_on_empty_config() {
        let project = Path::new("/tmp/example-project");
        let output = upsert_project_trust("", project, TrustLevel::Trusted).unwrap();
        let doc = output.parse::<DocumentMut>().unwrap();
        let key = project.to_string_lossy().to_string();

        assert_eq!(
            doc["projects"][key.as_str()]["trust_level"].as_str(),
            Some("trusted")
        );
        assert!(doc["projects"][key.as_str()].is_table());
    }

    #[test]
    fn upsert_updates_existing_project() {
        let project = Path::new("/tmp/example-project");
        let input = r#"
[projects."/tmp/example-project"]
trust_level = "trusted"
"#;
        let output = upsert_project_trust(input, project, TrustLevel::Untrusted).unwrap();
        let doc = output.parse::<DocumentMut>().unwrap();
        let key = project.to_string_lossy().to_string();

        assert_eq!(
            doc["projects"][key.as_str()]["trust_level"].as_str(),
            Some("untrusted")
        );
    }

    #[test]
    fn upsert_migrates_inline_projects_table() {
        let project = Path::new("/tmp/new-worktree");
        let input = r#"
projects = { "/tmp/existing" = { trust_level = "trusted" } }
"#;
        let output = upsert_project_trust(input, project, TrustLevel::Trusted).unwrap();
        let doc = output.parse::<DocumentMut>().unwrap();
        let new_key = project.to_string_lossy().to_string();

        assert_eq!(
            doc["projects"]["/tmp/existing"]["trust_level"].as_str(),
            Some("trusted")
        );
        assert_eq!(
            doc["projects"][new_key.as_str()]["trust_level"].as_str(),
            Some("trusted")
        );
        assert!(doc["projects"]["/tmp/existing"].is_table());
    }
}
