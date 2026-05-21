use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfLabConfig {
    pub spec_dir: PathBuf,
    pub local_fixtures_dir: PathBuf,
    pub output_dir: PathBuf,
    pub baseline_manifest: Option<PathBuf>,
    pub profiles: Vec<String>,
    pub default_engine: String,
    pub strict: bool,
    #[serde(default = "default_synthetic_enabled")]
    pub synthetic_enabled: bool,
    #[serde(default = "default_synthetic_seed")]
    pub synthetic_seed: u64,
}

impl PerfLabConfig {
    pub fn load(repo_root: &Path, config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(config_path)?;
        let mut cfg: Self = ron::from_str(&content)?;
        cfg.spec_dir = absolutize(repo_root, &cfg.spec_dir);
        cfg.local_fixtures_dir = absolutize(repo_root, &cfg.local_fixtures_dir);
        cfg.output_dir = absolutize(repo_root, &cfg.output_dir);
        cfg.baseline_manifest = cfg
            .baseline_manifest
            .as_ref()
            .map(|path| absolutize(repo_root, path));
        Ok(cfg)
    }
}

fn absolutize(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn default_synthetic_enabled() -> bool {
    true
}

fn default_synthetic_seed() -> u64 {
    1337
}
