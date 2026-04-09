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
config: !include data/base.yaml

# Specific nested field
city: !include [data/tags.yaml, "address/city"]

# Sequence index: second user's name
second_user: !include [data/users.yaml, "1/name"]
```

### `!merge`

```yaml
# Merge mappings (deep, left-to-right)
merged_db: !merge [data/base.yaml, data/overrides.yaml, data/extra.yaml]

# Merge sequences (concatenated)
all_items: !merge [data/items.yaml, data/more_items.yaml]

# Root-level merge
!merge [data/base.yaml, data/overrides.yaml]
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

Pre-built binaries are available on the [Releases](https://github.com/mivnixv/yamlext/releases) page.

### Build from source

```sh
cargo build --release
# binary at target/release/yamlext
```

### `!include` notes

- Paths are relative to the file containing the tag (so included files can have their own relative includes)

### `!merge` notes

- Mappings: later files override earlier ones, recursively
- Sequences: items are appended in order
- Merging a mapping with a sequence is an error

### Examples

See the [`examples/`](examples/) directory:

| File | Description |
|------|-------------|
| [main.yaml](examples/main.yaml) | All tags used as nested values |
| [merge.yaml](examples/merge.yaml) | Merge examples (mappings and sequences) |
| [root_level_merge.yaml](examples/root_level_merge.yaml) | Root-level merge (entire document is the result) |
| [data/base.yaml](examples/data/base.yaml) | Base database config |
| [data/overrides.yaml](examples/data/overrides.yaml) | Overrides for database host/port |
| [data/extra.yaml](examples/data/extra.yaml) | Extra database fields |
| [data/tags.yaml](examples/data/tags.yaml) | Name and address data |
| [data/items.yaml](examples/data/items.yaml) | First sequence of items |
| [data/more_items.yaml](examples/data/more_items.yaml) | Second sequence of items |
| [data/users.yaml](examples/data/users.yaml) | Sequence of user objects |
