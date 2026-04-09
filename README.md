# yamlext ![Main Build](https://github.com/mivnixv/yamlext/actions/workflows/test.yml/badge.svg?branch=main)

A CLI tool that extends YAML with `!include` and `!merge` custom tags.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/mivnixv/yamlext/main/install.sh | sh
```

## Usage

```sh
yamlext input.yaml
yamlext input.yaml --base-dir /path/to/base
```

## Custom Tags

### `!include`

```yaml
# Entire file
config: !include path/to/file.yaml

# Specific nested field
city: !include [path/to/file.yaml, "address/city"]
```

### `!merge`

```yaml
# Merge mappings (deep, left-to-right)
merged: !merge [base.yaml, overrides.yaml, extras.yaml]

# Merge sequences (concatenated)
all_items: !merge [list1.yaml, list2.yaml]

# Root-level merge
!merge [base.yaml, overrides.yaml]
```

---

## Advanced

### Install options

Override install directory or pin a version:

```sh
curl -fsSL https://raw.githubusercontent.com/mivnixv/yamlext/main/install.sh | INSTALL_DIR=~/.local/bin sh
curl -fsSL https://raw.githubusercontent.com/mivnixv/yamlext/main/install.sh | VERSION=v1.0.0 sh
```

### Manual download

Pre-built binaries are available on the [Releases](https://github.com/mivnixv/yamlext/releases) page:

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `yamlext-linux-x86_64` |
| Linux aarch64 | `yamlext-linux-aarch64` |
| macOS x86_64 | `yamlext-macos-x86_64` |
| macOS aarch64 (Apple Silicon) | `yamlext-macos-aarch64` |
| Windows x86_64 | `yamlext-windows-x86_64.exe` |

Each release includes `checksums.txt` for verification:

```sh
sha256sum -c checksums.txt
```

### Build from source

```sh
cargo build --release
# binary at target/release/yamlext
```

### `!include` notes

- Paths are relative to the file containing the tag (so included files can have their own relative includes)
- Field paths support sequence indices: `!include [file.yaml, "items/0/name"]`

### `!merge` notes

- Mappings: later files override earlier ones, recursively
- Sequences: items are appended in order
- Merging a mapping with a sequence is an error

### Examples

See the [`examples/`](examples/) directory:

| File | Description |
|------|-------------|
| [main.yaml](examples/main.yaml) | All tags used as nested values |
| [merge_mappings.yaml](examples/merge_mappings.yaml) | Root-level merge of mappings |
| [merge_sequences.yaml](examples/merge_sequences.yaml) | Root-level merge of sequences |
