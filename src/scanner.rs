use crate::artifact::{Artifact, ArtifactKind, AuditManifest};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn scan(root: impl AsRef<Path>) -> io::Result<AuditManifest> {
    let root = root.as_ref();
    let canonical_root = root.canonicalize()?;

    let mut artifacts = Vec::new();
    collect_artifacts(&canonical_root, &canonical_root, &mut artifacts)?;
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(AuditManifest {
        root: canonical_root,
        artifacts,
    })
}

fn collect_artifacts(root: &Path, current: &Path, artifacts: &mut Vec<Artifact>) -> io::Result<()> {
    let mut entries = fs::read_dir(current)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_artifacts(root, &path, artifacts)?;
        } else if metadata.is_file() {
            let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            artifacts.push(Artifact {
                kind: classify(&relative_path),
                path: normalize_path(relative_path),
                size_bytes: metadata.len(),
            });
        }
    }

    Ok(())
}

pub fn classify(path: &Path) -> ArtifactKind {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let path_text = path.to_string_lossy().to_ascii_lowercase();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match extension.as_str() {
        "yaml" | "yml" | "toml" => ArtifactKind::Config,
        "jsonl" => ArtifactKind::JsonLog,
        "log" | "txt" => ArtifactKind::TextLog,
        "png" | "jpg" | "jpeg" | "webp" => ArtifactKind::Image,
        "md" | "html" => ArtifactKind::Report,
        "json" => {
            if path_text.contains("result")
                || path_text.contains("output")
                || path_text.contains("eval")
            {
                ArtifactKind::ResultJson
            } else if file_name.contains("config") || path_text.contains("config") {
                ArtifactKind::Config
            } else {
                ArtifactKind::ResultJson
            }
        }
        _ => ArtifactKind::Other,
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let normalized = path.to_string_lossy().replace('\\', "/");
    PathBuf::from(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_artifacts() {
        assert_eq!(
            classify(Path::new("configs/run.yaml")),
            ArtifactKind::Config
        );
        assert_eq!(classify(Path::new("logs/run.jsonl")), ArtifactKind::JsonLog);
        assert_eq!(
            classify(Path::new("outputs/result.json")),
            ArtifactKind::ResultJson
        );
        assert_eq!(
            classify(Path::new("outputs/sample.png")),
            ArtifactKind::Image
        );
    }
}
