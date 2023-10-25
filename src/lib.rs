use itertools::Itertools;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AppError {
    #[error("Unity is not officially supported in your OS")]
    UnsupportedOS,

    #[error("Could not find unity installation base dir: {0}")]
    BasedirFindIssue(String),

    #[error("Could not find any unity installations")]
    NoUnityInstallations,

    #[error("Could not access unity installations directory")]
    NoAccessHubInstallations,

    #[error("Invalid project version file detected")]
    InvalidProjectVersionFile,

    #[error("Could not find ProjectVersion.txt for current project")]
    ProjectVersionFileNotFound,

    #[error("Could not read ProjectVersion.txt for current project")]
    ProjectVersionFileUnreadable,

    #[error("Could not find UnityYAMLMerge tool")]
    YamlToolNotFound,
}

// TODO: reconsider hashmap?
#[derive(Debug, Clone)]
pub(crate) struct UnityInstallation {
    pub version: String,
    pub path: PathBuf,
}
type UnityInstallationCollection = Vec<UnityInstallation>;

/// Simple enum for marking the operating system.
/// Note that it only includes the platforms unity supports.
#[derive(Debug, Clone, Copy)]
pub(crate) enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
}

// TODO: test
/// Get the current OS based on the version of unity used

pub(crate) fn get_current_os() -> Result<OperatingSystem, AppError> {
    match std::env::consts::OS {
        "windows" => Ok(OperatingSystem::Windows),
        "macos" => Ok(OperatingSystem::MacOS),
        "linux" => Ok(OperatingSystem::Linux),
        _ => Err(AppError::UnsupportedOS),
    }
}

// TODO: test
pub(crate) fn get_unityhub_base_app_path(os: OperatingSystem) -> Result<PathBuf, AppError> {
    match os {
        OperatingSystem::Windows => Ok(PathBuf::from(
            std::env::var("PROGRAMFILES").unwrap_or(String::from("C:\\Program Files")),
        )),
        OperatingSystem::MacOS => Ok(PathBuf::from("/Applications")),
        OperatingSystem::Linux => match std::env::var("HOME") {
            Ok(value) => Ok(PathBuf::from(value)),
            Err(err) => Err(AppError::BasedirFindIssue(err.to_string())),
        },
    }
}

/// Gets the full path for hub-based unity installations
pub(crate) fn get_unityhub_base_installations_path(
    os: OperatingSystem,
) -> Result<PathBuf, AppError> {
    // Technically this should've been "more" os-specific,
    // but Unity Hub keeps the same installation scheme across all platforms, so we generalize
    let mut path = get_unityhub_base_app_path(os)?;
    path.push("Unity");
    path.push("Hub");
    path.push("Editor");

    Ok(path)
}

// TODO: test
pub(crate) fn get_unityhub_installations(
    os: OperatingSystem,
) -> Result<UnityInstallationCollection, AppError> {
    let mut installations = UnityInstallationCollection::new();

    let base_installation_path = get_unityhub_base_installations_path(os)?;
    let readdir = std::fs::read_dir(base_installation_path)
        .map_err(|_| AppError::NoAccessHubInstallations)?;
    for entry in readdir.filter_map(Result::ok) {
        let editor_path: PathBuf = entry.path();
        if !editor_path.is_dir() {
            continue;
        }

        let unity_exe_path = editor_path.join("Editor").join(match os {
            OperatingSystem::Windows => "Unity.exe",
            OperatingSystem::MacOS => "MacOS/Unity",
            OperatingSystem::Linux => "Unity",
        });
        if unity_exe_path.try_exists().is_err() || !unity_exe_path.is_file() {
            continue;
        }

        // TODO: actual IO errors
        let version_id = editor_path
            .file_name()
            .expect("editor version should be a valid string")
            .to_str()
            .expect("editor version should unicode-decodeable")
            .into();
        installations.push(UnityInstallation {
            version: version_id,
            path: editor_path,
        });
    }

    Ok(installations)
}

/// Parses unity-generated `ProjectVersion.txt` files
pub(crate) fn parse_project_version_file(file: &str) -> Result<String, AppError> {
    const SEPERATOR: &str = "m_EditorVersion: ";
    let line = file
        .lines()
        .find(|&s| s.starts_with(SEPERATOR))
        .ok_or(AppError::InvalidProjectVersionFile)?;

    Ok(line[SEPERATOR.len()..].to_owned())
}

/// Locate a unity-generated `ProjectVersion.txt` files
/// TODO test
pub(crate) fn locate_project_version_file(workdir: &Path) -> Result<PathBuf, AppError> {
    let version_file = workdir.join("ProjectSettings/ProjectVersion.txt");

    if !version_file.exists() || !version_file.is_file() {
        Err(AppError::ProjectVersionFileNotFound)
    } else {
        Ok(version_file)
    }
}

/// Read a project's version (via `ProjectVersion.txt`)
pub(crate) fn read_project_version(workdir: &Path) -> Result<String, AppError> {
    let version_file = locate_project_version_file(workdir)?;
    let contents = std::fs::read_to_string(version_file)
        .map_err(|_| AppError::ProjectVersionFileUnreadable)?;

    parse_project_version_file(&contents)
}

/// Select the most appropriate installation
pub(crate) fn choose_best_installation<'a>(
    workdir: &'a Path,
    installations: &'a UnityInstallationCollection,
) -> Result<&'a UnityInstallation, AppError> {
    // try reading it from current directory
    println!("Attempting to probe CWD as project...");
    let project_editor_version = read_project_version(workdir);
    match project_editor_version {
        Ok(version) => return Ok(installations.iter().find(|x| x.version == version).unwrap()),
        Err(AppError::ProjectVersionFileNotFound) => {
            println!("CWD is not a project, choosing latest version...");
        }
        Err(err) => return Err(err),
    };

    // Choose latest installation instead
    installations
        .iter()
        .sorted_by_key(|&x| &x.version)
        .next_back()
        .ok_or(AppError::NoUnityInstallations)
}

/// Get the path of `UnityYAMLMerge` for an installation
pub(crate) fn get_yamltool(os: OperatingSystem, installation: &Path) -> Result<PathBuf, AppError> {
    let yamltool: PathBuf = match os {
        OperatingSystem::Windows => installation.join("Editor/Data/Tools/UnityYAMLMerge.exe"),
        OperatingSystem::MacOS => todo!(),
        OperatingSystem::Linux => todo!(),
    };

    if !yamltool.exists() {
        return Err(AppError::YamlToolNotFound);
    }

    Ok(yamltool)
}

/// Runs the executable based on given strings
// TODO: support "located" installations
// TODO: maybe support UnityHub alternatives?
pub fn run(args: &[String]) -> anyhow::Result<i32> {
    let os = get_current_os()?;

    let installations = get_unityhub_installations(os)?;
    if installations.is_empty() {
        return Err(AppError::NoUnityInstallations.into());
    }
    for installation in &installations {
        println!("Installation detected: {installation:?}");
    }

    let workdir = std::env::current_dir().unwrap();
    println!("Working directory: {workdir:?}");

    let installation = choose_best_installation(&workdir, &installations)?;
    println!("Selected installation: {installation:?}");

    let yamltool: PathBuf = get_yamltool(os, &installation.path)?;
    println!("Selected yamltool: {yamltool:?}");

    if std::env::var("UYAMLT_DRY_RUN").is_ok() {
        return Ok(0);
    }

    println!("passing through...");
    let process_result = std::process::Command::new(std::fs::canonicalize(yamltool).unwrap())
        .args(args)
        .spawn()?
        .wait()?;

    Ok(process_result.code().expect("Process terminated by signal"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_project_version_minimal() {
        let minimal_project_version_file = r#"m_EditorVersion: 2022.3.11f1
"#;
        let result = parse_project_version_file(minimal_project_version_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2022.3.11f1");
    }

    #[test]
    fn test_parse_project_version_actual() {
        let sample_project_version_file = r#"m_EditorVersion: 2022.3.11f1
m_EditorVersionWithRevision: 2022.3.11f1 (d00248457e15)
"#;
        let result = parse_project_version_file(sample_project_version_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2022.3.11f1");
    }

    #[test]
    fn test_parse_project_version_invalid() {
        let invalid_project_file = r#"pver: 2022.3.11f1 (d00248457e15)
"#;
        let result = parse_project_version_file(invalid_project_file);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), AppError::InvalidProjectVersionFile);
    }
}
