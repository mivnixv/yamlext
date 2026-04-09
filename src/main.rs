use anyhow::{bail, Context, Result};
use clap::Parser;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// YAML processor with !include and !merge custom tag support.
///
/// Custom tags:
///   !include path/to/file.yaml
///   !include [path/to/file.yaml, "nested/field/path"]
///   !merge [path/to/file1.yaml, path/to/file2.yaml, ...]
#[derive(Parser)]
#[command(name = "yamlext", version, about)]
struct Cli {
    /// Input YAML file to process (use '-' for stdin)
    input: String,

    /// Base directory for resolving relative paths (defaults to input file's directory)
    #[arg(short, long)]
    base_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (content, base_dir) = if cli.input == "-" {
        let content = std::io::read_to_string(std::io::stdin())?;
        let base_dir = cli
            .base_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        (content, base_dir)
    } else {
        let path = PathBuf::from(&cli.input);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read '{}'", cli.input))?;
        let base_dir = cli.base_dir.unwrap_or_else(|| {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        });
        (content, base_dir)
    };

    let label = if cli.input == "-" { "<stdin>".to_string() } else { cli.input.clone() };
    let value: Value = parse_yaml(&content, &label)?;
    let resolved = resolve(value, &base_dir)?;

    print!("{}", serde_yaml::to_string(&resolved)?);

    Ok(())
}

/// Recursively walk and resolve custom tags in a YAML value.
fn resolve(value: Value, base_dir: &Path) -> Result<Value> {
    match value {
        Value::Tagged(tagged) => {
            let tag = tagged.tag.to_string();
            let inner = tagged.value;
            match tag.as_str() {
                "!include" => handle_include(inner, base_dir),
                "!merge" => handle_merge(inner, base_dir),
                _ => {
                    // Unknown tag: resolve inner value but preserve the tag.
                    let resolved = resolve(inner, base_dir)?;
                    Ok(Value::Tagged(Box::new(serde_yaml::value::TaggedValue {
                        tag: tagged.tag,
                        value: resolved,
                    })))
                }
            }
        }
        Value::Mapping(map) => {
            let mut new_map = serde_yaml::Mapping::new();
            for (k, v) in map {
                new_map.insert(k, resolve(v, base_dir)?);
            }
            Ok(Value::Mapping(new_map))
        }
        Value::Sequence(seq) => {
            let resolved: Result<Vec<Value>> =
                seq.into_iter().map(|v| resolve(v, base_dir)).collect();
            Ok(Value::Sequence(resolved?))
        }
        other => Ok(other),
    }
}

// ---------------------------------------------------------------------------
// !include
// ---------------------------------------------------------------------------

/// Handle the !include tag.
///
/// Forms:
///   !include path/to/file.yaml            -> entire file
///   !include [path/to/file.yaml, "a/b/c"] -> nested field a.b.c from file
fn handle_include(value: Value, base_dir: &Path) -> Result<Value> {
    match value {
        Value::String(path_str) => {
            let file_path = resolve_path(base_dir, &path_str);
            load_yaml(&file_path, base_dir)
        }
        Value::Sequence(seq) => {
            if seq.len() != 2 {
                bail!(
                    "!include sequence must have exactly 2 elements: [path, field_path], got {}",
                    seq.len()
                );
            }
            let path_str = as_string(&seq[0], "!include path")?;
            let field_path = as_string(&seq[1], "!include field path")?;
            let file_path = resolve_path(base_dir, &path_str);
            let loaded = load_yaml(&file_path, base_dir)?;
            navigate(&loaded, &field_path)
                .with_context(|| format!("navigating to '{}' in '{}'", field_path, path_str))
        }
        _ => bail!("!include value must be a string or [path, field_path] sequence"),
    }
}

/// Navigate into a YAML value following a slash-separated field path.
///
/// Supports:
///   - mapping keys:    "a/b/c"
///   - sequence index:  "items/0/name"
fn navigate(value: &Value, field_path: &str) -> Result<Value> {
    let parts: Vec<&str> = field_path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut current = value;
    for part in &parts {
        match current {
            Value::Mapping(map) => {
                let key = Value::String(part.to_string());
                current = map
                    .get(&key)
                    .with_context(|| format!("key '{}' not found in mapping", part))?;
            }
            Value::Sequence(seq) => {
                let idx: usize = part
                    .parse()
                    .with_context(|| format!("expected numeric index, got '{}'", part))?;
                current = seq
                    .get(idx)
                    .with_context(|| format!("index {} out of bounds (len={})", idx, seq.len()))?;
            }
            _ => bail!("cannot navigate into scalar value at segment '{}'", part),
        }
    }
    Ok(current.clone())
}

// ---------------------------------------------------------------------------
// !merge
// ---------------------------------------------------------------------------

/// Handle the !merge tag.
///
/// Form:
///   !merge [path/to/file1.yaml, path/to/file2.yaml, ...]
///
/// All files must be the same collection type:
///   - Mappings are deep-merged left-to-right (later files override earlier).
///   - Sequences are concatenated left-to-right.
fn handle_merge(value: Value, base_dir: &Path) -> Result<Value> {
    let paths = match value {
        Value::Sequence(seq) => seq,
        _ => bail!("!merge value must be a sequence of file paths"),
    };

    if paths.is_empty() {
        bail!("!merge requires at least one path");
    }

    let mut result: Option<Value> = None;

    for path_val in paths {
        let path_str = as_string(&path_val, "!merge path")?;
        let file_path = resolve_path(base_dir, &path_str);
        let loaded = load_yaml(&file_path, base_dir)
            .with_context(|| format!("loading '{}'", path_str))?;

        result = Some(match result {
            None => loaded,
            Some(existing) => deep_merge(existing, loaded)
                .with_context(|| format!("merging '{}'", path_str))?,
        });
    }

    Ok(result.unwrap())
}

/// Deep-merge `other` into `base`.
///
/// - Mappings: keys from `other` override `base`; nested mappings are recursively merged.
/// - Sequences: items from `other` are appended after `base`.
/// - Mixed types: error.
fn deep_merge(base: Value, other: Value) -> Result<Value> {
    match (base, other) {
        (Value::Mapping(mut base_map), Value::Mapping(other_map)) => {
            for (k, v) in other_map {
                let merged = if let Some(base_v) = base_map.remove(&k) {
                    deep_merge(base_v, v)?
                } else {
                    v
                };
                base_map.insert(k, merged);
            }
            Ok(Value::Mapping(base_map))
        }
        (Value::Sequence(mut base_seq), Value::Sequence(other_seq)) => {
            base_seq.extend(other_seq);
            Ok(Value::Sequence(base_seq))
        }
        // Mapping vs sequence is a hard error; everything else (scalar/scalar,
        // scalar/collection, etc.) lets `other` overwrite `base`.
        (Value::Mapping(_), Value::Sequence(_)) | (Value::Sequence(_), Value::Mapping(_)) => {
            bail!("cannot merge a mapping with a sequence");
        }
        (_base, other) => Ok(other),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_yaml(content: &str, label: &str) -> Result<Value> {
    serde_yaml::from_str(content)
        .map_err(|e| anyhow::anyhow!("{label}: {e}"))
}

fn load_yaml(path: &Path, base_dir: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read '{}'", path.display()))?;
    let value: Value = parse_yaml(&content, &path.display().to_string())?;
    // Recursively resolve custom tags in included/merged files,
    // using that file's own directory as the new base.
    let new_base = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| base_dir.to_path_buf());
    resolve(value, &new_base)
}

fn resolve_path(base_dir: &Path, path_str: &str) -> PathBuf {
    let p = Path::new(path_str);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

fn as_string(value: &Value, label: &str) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => bail!("{} must be a string, got {}", label, type_name(value)),
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Sequence(_) => "sequence",
        Value::Mapping(_) => "mapping",
        Value::Tagged(_) => "tagged",
    }
}
