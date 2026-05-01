use crate::config::PerfLabConfig;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    pub id: String,
    pub profile: String,
    pub source_path: PathBuf,
    pub bytes: usize,
    pub sha256: String,
}

#[derive(Debug, Default)]
pub struct ManifestDrift {
    pub missing_ids: Vec<String>,
    pub unexpected_ids: Vec<String>,
    pub changed_ids: Vec<String>,
}

impl ManifestDrift {
    pub fn is_empty(&self) -> bool {
        self.missing_ids.is_empty() && self.unexpected_ids.is_empty() && self.changed_ids.is_empty()
    }
}

pub fn discover_workloads(
    _repo_root: &Path,
    config: &PerfLabConfig,
) -> Result<Vec<Workload>, Box<dyn std::error::Error>> {
    if config.synthetic_enabled {
        materialize_synthetic_fixtures(&config.local_fixtures_dir, config.synthetic_seed)?;
    }

    let mut workloads = Vec::new();
    workloads.extend(spec_workloads(&config.spec_dir)?);
    workloads.extend(local_fixture_workloads(&config.local_fixtures_dir)?);
    workloads.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(workloads)
}

pub fn write_manifest(
    path: &Path,
    workloads: &[Workload],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(workloads)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_manifest(path: &Path) -> Result<Vec<Workload>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let manifest = serde_json::from_str::<Vec<Workload>>(&content)?;
    Ok(manifest)
}

pub fn compute_manifest_drift(expected: &[Workload], current: &[Workload]) -> ManifestDrift {
    let expected_by_id: HashMap<&str, &Workload> =
        expected.iter().map(|w| (w.id.as_str(), w)).collect();
    let current_by_id: HashMap<&str, &Workload> = current.iter().map(|w| (w.id.as_str(), w)).collect();

    let mut drift = ManifestDrift::default();

    for id in expected_by_id.keys() {
        if !current_by_id.contains_key(id) {
            drift.missing_ids.push((*id).to_string());
        }
    }

    for id in current_by_id.keys() {
        if !expected_by_id.contains_key(id) {
            drift.unexpected_ids.push((*id).to_string());
        }
    }

    for id in expected_by_id.keys() {
        if let (Some(expected_item), Some(current_item)) =
            (expected_by_id.get(id), current_by_id.get(id))
        {
            if expected_item.sha256 != current_item.sha256
                || expected_item.bytes != current_item.bytes
                || expected_item.profile != current_item.profile
            {
                drift.changed_ids.push((*id).to_string());
            }
        }
    }

    drift.missing_ids.sort();
    drift.unexpected_ids.sort();
    drift.changed_ids.sort();
    drift
}

fn spec_workloads(spec_dir: &Path) -> Result<Vec<Workload>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(spec_dir)? {
        let entry = entry?;
        let path = entry.path();
        let is_json = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if !is_json {
            continue;
        }

        let bytes = std::fs::read(&path)?;
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("invalid UTF-8 in spec file name")?
            .to_string();
        out.push(Workload {
            id: format!("spec:{file_stem}"),
            profile: infer_profile(&file_stem),
            source_path: path,
            bytes: bytes.len(),
            sha256: hash_bytes(&bytes),
        });
    }
    Ok(out)
}

fn local_fixture_workloads(fixtures_root: &Path) -> Result<Vec<Workload>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    if !fixtures_root.exists() {
        return Ok(out);
    }

    for tier in ["small", "medium", "large", "pathological"] {
        let tier_dir = fixtures_root.join(tier);
        if !tier_dir.exists() {
            continue;
        }

        for entry in std::fs::read_dir(&tier_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => continue,
            };
            if file_name.starts_with('.') {
                continue;
            }

            let bytes = std::fs::read(&path)?;
            out.push(Workload {
                id: format!("fixture:{tier}:{file_name}"),
                profile: String::from("marco-extensions"),
                source_path: path,
                bytes: bytes.len(),
                sha256: hash_bytes(&bytes),
            });
        }
    }

    Ok(out)
}

fn infer_profile(stem: &str) -> String {
    if stem.contains("commonmark") {
        String::from("commonmark-core")
    } else if stem.contains("gfm") {
        String::from("gfm-core")
    } else {
        String::from("marco-extensions")
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn materialize_synthetic_fixtures(
    fixtures_root: &Path,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for tier in ["small", "medium", "large", "pathological"] {
        std::fs::create_dir_all(fixtures_root.join(tier))?;
    }

    let mut state = seed;
    let small_doc = synthetic_small_doc(&mut state);
    let medium_doc = synthetic_medium_doc(&mut state);
    let large_doc = synthetic_large_doc(&mut state);
    let pathological_doc = synthetic_pathological_doc(&mut state);

    write_if_missing(
        &fixtures_root.join("small/generated-synthetic.md"),
        &small_doc,
    )?;
    write_if_missing(
        &fixtures_root.join("medium/generated-synthetic.md"),
        &medium_doc,
    )?;
    write_if_missing(
        &fixtures_root.join("large/generated-synthetic.md"),
        &large_doc,
    )?;
    write_if_missing(
        &fixtures_root.join("pathological/generated-synthetic.md"),
        &pathological_doc,
    )?;

    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        return Ok(());
    }
    std::fs::write(path, content)?;
    Ok(())
}

fn synthetic_small_doc(state: &mut u64) -> String {
    let title = sample_word(state);
    let item_a = sample_word(state);
    let item_b = sample_word(state);
    format!(
        "# {title}\n\nA compact synthetic fixture for baseline runs.\n\n- {item_a}\n- {item_b}\n"
    )
}

fn synthetic_medium_doc(state: &mut u64) -> String {
    let mut out = String::from("## Synthetic Medium Fixture\n\n");
    for i in 0..24 {
        let w1 = sample_word(state);
        let w2 = sample_word(state);
        out.push_str(&format!(
            "Paragraph {i}: {w1} **{w2}** with [link](https://example.com/{w1}-{w2}).\n\n"
        ));
    }
    out
}

fn synthetic_large_doc(state: &mut u64) -> String {
    let mut out = String::from("## Synthetic Large Fixture\n\n");
    for i in 0..500 {
        let w1 = sample_word(state);
        let w2 = sample_word(state);
        let w3 = sample_word(state);
        out.push_str(&format!(
            "{i}. {w1} {w2} {w3} `code-{i}` [ref](https://example.org/{w1}/{w2})\n"
        ));
    }
    out
}

fn synthetic_pathological_doc(state: &mut u64) -> String {
    let mut out = String::from("## Synthetic Pathological Fixture\n\n");
    for depth in 1..=64 {
        let marker = "*".repeat(depth);
        let word = sample_word(state);
        out.push_str(&format!("{marker} {word}\n"));
    }

    out.push_str("\n");
    for i in 0..128 {
        let open = "[".repeat((i % 7) + 1);
        let close = "]".repeat((i % 5) + 1);
        out.push_str(&format!("line-{i}: {open}unbalanced{close}\n"));
    }

    out
}

fn sample_word(state: &mut u64) -> &'static str {
    const WORDS: &[&str] = &[
        "alpha",
        "bravo",
        "charlie",
        "delta",
        "echo",
        "foxtrot",
        "golf",
        "hotel",
        "india",
        "juliet",
        "kilo",
        "lima",
        "mike",
        "november",
        "oscar",
        "papa",
        "quebec",
        "romeo",
        "sierra",
        "tango",
        "uniform",
        "victor",
        "whiskey",
        "xray",
        "yankee",
        "zulu",
    ];

    let idx = (lcg_next(state) % WORDS.len() as u64) as usize;
    WORDS[idx]
}

fn lcg_next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}
