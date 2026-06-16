use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactKind {
    Config,
    JsonLog,
    TextLog,
    ResultJson,
    Image,
    Report,
    Other,
}

/// 实验目录下扫描到的单个文件
#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: PathBuf,
    pub kind: ArtifactKind,
    pub size_bytes: u64,
}

/// 扫描阶段产出的稳定文件清单
#[derive(Debug, Clone)]
pub struct AuditManifest {
    pub root: PathBuf,
    pub artifacts: Vec<Artifact>,
}

impl AuditManifest {
    /// 按文件类型统计数量
    pub fn count_by_kind(&self) -> BTreeMap<ArtifactKind, usize> {
        let mut counts = BTreeMap::new();
        for artifact in &self.artifacts {
            *counts.entry(artifact.kind.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn total_size_bytes(&self) -> u64 {
        self.artifacts
            .iter()
            .map(|artifact| artifact.size_bytes)
            .sum()
    }
}

impl ArtifactKind {
    /// 统一输出中的类型名称
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactKind::Config => "config",
            ArtifactKind::JsonLog => "json_log",
            ArtifactKind::TextLog => "text_log",
            ArtifactKind::ResultJson => "result_json",
            ArtifactKind::Image => "image",
            ArtifactKind::Report => "report",
            ArtifactKind::Other => "other",
        }
    }
}
