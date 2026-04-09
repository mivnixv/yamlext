use anyhow::{bail, Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "yamlext", about = "YAML processor with !include and !merge tags", version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Input YAML file to process
    input: PathBuf,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Base directory for resolving include/merge paths (defaults to input file's directory)
    #[arg(long)]
    base_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let base_dir = cli.base_dir.clone().unwrap_or_else(|| {
        cli.input
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf()
    });

    eprintln!("[yamlext] processing {}", cli.input.display());

    let content = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("Failed to read {:?}", cli.input))?;

    let mut seen = HashSet::new();
    seen.insert(
        cli.input
            .canonicalize()
            .unwrap_or(cli.input.clone()),
    );

    let result = process(&content, &base_dir, &mut seen)?;

    match &cli.output {
        Some(path) => {
            std::fs::write(path, &result)
                .with_context(|| format!("Failed to write {:?}", path))?;
            eprintln!("[yamlext] written to {}", path.display());
        }
        None => print!("{}", result),
    }

    eprintln!("[yamlext] done ({} bytes)", result.len());
    Ok(())
}

/// Process a YAML string, resolving all !include and !merge tags.
/// `base_dir` is used to resolve relative paths in tags.
fn process(content: &str, base_dir: &Path, seen: &mut HashSet<PathBuf>) -> Result<String> {
    let mut out = String::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];

        // Match: `!include ...` (standalone, no key)
        if let Some(rest) = trimmed.strip_prefix("!include ") {
            let resolved = resolve_include(rest.trim(), base_dir, seen, indent)?;
            out.push_str(&resolved);
            out.push('\n');
        // Match: `!merge ...` (standalone, no key)
        } else if let Some(rest) = trimmed.strip_prefix("!merge ") {
            let resolved = resolve_merge(rest.trim(), base_dir, seen, indent)?;
            out.push_str(&resolved);
            out.push('\n');
        // Match: `key: !include ...`
        } else if let Some((key, rest)) = split_key_tag(trimmed, "!include ") {
            let child_indent = format!("{}  ", indent);
            let resolved = resolve_include(rest.trim(), base_dir, seen, &child_indent)?;
            emit_key_value(&mut out, indent, key, &resolved);
        // Match: `key: !merge ...`
        } else if let Some((key, rest)) = split_key_tag(trimmed, "!merge ") {
            let child_indent = format!("{}  ", indent);
            let resolved = resolve_merge(rest.trim(), base_dir, seen, &child_indent)?;
            emit_key_value(&mut out, indent, key, &resolved);
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    // Remove trailing newline added by last push
    if out.ends_with('\n') && !content.ends_with('\n') {
        out.pop();
    }

    Ok(out)
}

/// Emit `indent + key: value\n` — inline if value is a single line, block otherwise.
fn emit_key_value(out: &mut String, indent: &str, key: &str, resolved: &str) {
    let is_single_line = !resolved.trim().contains('\n');
    if is_single_line {
        // scalar: `key: value`
        out.push_str(indent);
        out.push_str(key);
        out.push_str(": ");
        out.push_str(resolved.trim());
        out.push('\n');
    } else {
        // mapping/sequence: key on its own line, content already indented
        out.push_str(indent);
        out.push_str(key);
        out.push_str(":\n");
        out.push_str(resolved);
        out.push('\n');
    }
}

/// If `line` contains `key: <tag>rest`, return `(key, rest)`. Otherwise None.
fn split_key_tag<'a>(line: &'a str, tag: &str) -> Option<(&'a str, &'a str)> {
    // Find `: !include ` or `: !merge ` etc.
    let needle = format!(": {}", tag);
    let pos = line.find(needle.as_str())?;
    let key = &line[..pos];
    let rest = &line[pos + needle.len()..];
    // key must not itself contain a colon (avoid matching inside strings)
    if key.contains(':') {
        return None;
    }
    Some((key, rest))
}

/// Parse the argument list for a tag. Handles:
///   file.yaml
///   [file.yaml, "path/key"]
///   [file1.yaml, file2.yaml, ...]
fn parse_args(raw: &str) -> Result<Vec<String>> {
    let raw = raw.trim();
    if raw.starts_with('[') {
        let inner = raw
            .strip_prefix('[')
            .unwrap()
            .strip_suffix(']')
            .with_context(|| format!("Unmatched '[' in tag argument: {raw}"))?;
        let parts: Vec<String> = inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(parts)
    } else {
        Ok(vec![raw
            .trim_matches('"')
            .trim_matches('\'')
            .to_string()])
    }
}

/// Load a YAML file, recursively resolving includes, and return its processed content.
fn load_file(path: &Path, seen: &mut HashSet<PathBuf>, base_dir: &Path) -> Result<String> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Cannot resolve path {:?}", path))?;

    if seen.contains(&canonical) {
        bail!("Circular include detected: {:?}", canonical);
    }
    seen.insert(canonical.clone());


    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {:?}", path))?;

    let file_dir = path.parent().unwrap_or(base_dir);
    let processed = process(&content, file_dir, seen)?;

    seen.remove(&canonical);
    Ok(processed)
}

/// Resolve `!include file.yaml` or `!include [file.yaml, "key/path"]`.
/// Returns indented output lines (without trailing newline).
fn resolve_include(
    raw: &str,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
    indent: &str,
) -> Result<String> {
    let args = parse_args(raw)?;

    let file_path = base_dir.join(&args[0]);
    let content = load_file(&file_path, seen, base_dir)?;

    let content = if args.len() > 1 {
        // Extract sub-key from the loaded YAML
        extract_path(&content, &args[1])
            .with_context(|| format!("Key path '{}' not found in {:?}", args[1], file_path))?
    } else {
        content
    };

    Ok(indent_content(&content, indent))
}

/// Resolve `!merge [file1.yaml, file2.yaml, ...]`.
/// Merges mappings (deep) or sequences (concat) depending on root type.
fn resolve_merge(
    raw: &str,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
    indent: &str,
) -> Result<String> {
    let args = parse_args(raw)?;
    if args.is_empty() {
        bail!("!merge requires at least one file");
    }

    // Load all files as serde_yaml Values
    let mut values: Vec<serde_yaml::Value> = args
        .iter()
        .map(|f| {
            let path = base_dir.join(f);
            let content = load_file(&path, seen, base_dir)?;
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML from {:?}", path))
        })
        .collect::<Result<_>>()?;

    if values.is_empty() {
        return Ok(String::new());
    }

    let first = values.remove(0);
    let merged = match first {
        serde_yaml::Value::Mapping(_) => {
            let mut acc = first;
            for v in values {
                deep_merge_mapping(&mut acc, v)?;
            }
            acc
        }
        serde_yaml::Value::Sequence(_) => {
            let mut acc = match first {
                serde_yaml::Value::Sequence(s) => s,
                _ => unreachable!(),
            };
            for v in values {
                match v {
                    serde_yaml::Value::Sequence(s) => acc.extend(s),
                    other => bail!("Cannot merge sequence with non-sequence: {:?}", other),
                }
            }
            serde_yaml::Value::Sequence(acc)
        }
        other => bail!("!merge only supports mappings or sequences, got: {:?}", other),
    };

    let yaml_str = serde_yaml::to_string(&merged)
        .context("Failed to serialize merged YAML")?;
    // serde_yaml adds a leading "---\n" in some versions; strip it
    let yaml_str = yaml_str.strip_prefix("---\n").unwrap_or(&yaml_str);

    Ok(indent_content(yaml_str.trim_end(), indent))
}

/// Deep merge src into dst (both must be Mappings).
fn deep_merge_mapping(dst: &mut serde_yaml::Value, src: serde_yaml::Value) -> Result<()> {
    match (dst, src) {
        (serde_yaml::Value::Mapping(d), serde_yaml::Value::Mapping(s)) => {
            for (k, v) in s {
                match d.get_mut(&k) {
                    Some(existing) if existing.is_mapping() && v.is_mapping() => {
                        deep_merge_mapping(existing, v)?;
                    }
                    _ => {
                        d.insert(k, v);
                    }
                }
            }
        }
        (dst, src) => {
            bail!(
                "Cannot deep-merge non-mapping values: {:?} and {:?}",
                dst,
                src
            );
        }
    }
    Ok(())
}

/// Extract a nested value from parsed YAML using a slash-separated key path.
/// Returns the value serialized back to YAML text.
fn extract_path(content: &str, path: &str) -> Result<String> {
    let root: serde_yaml::Value =
        serde_yaml::from_str(content).context("Failed to parse YAML for path extraction")?;

    let mut current = &root;
    for key in path.split('/') {
        current = current
            .get(key)
            .with_context(|| format!("Key '{}' not found (in path '{}')", key, path))?;
    }

    let out = serde_yaml::to_string(current).context("Failed to serialize extracted value")?;
    let out = out.strip_prefix("---\n").unwrap_or(&out);
    Ok(out.trim_end().to_string())
}

/// Indent every line of `content` with `prefix`.
/// The first line is NOT prefixed (it will be placed after the tag on the same line's indent).
fn indent_content(content: &str, prefix: &str) -> String {
    let mut lines = content.lines();
    let mut result = String::new();

    if let Some(first) = lines.next() {
        result.push_str(prefix);
        result.push_str(first);
        for line in lines {
            result.push('\n');
            if line.is_empty() {
                // Don't add trailing spaces on blank lines
                result.push_str(line);
            } else {
                result.push_str(prefix);
                result.push_str(line);
            }
        }
    }

    result
}
