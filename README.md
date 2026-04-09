# yamlext

A CLI tool that extends YAML with `!include` and `!merge` custom tags.

## Install

### curl (Linux and macOS)

```sh
curl -fsSL https://raw.githubusercontent.com/mivnixv/yamlext/main/install.sh | sh
```

Installs to `/usr/local/bin` by default. Override with `INSTALL_DIR`:

```sh
curl -fsSL https://raw.githubusercontent.com/mivnixv/yamlext/main/install.sh | INSTALL_DIR=~/.local/bin sh
```

Pin to a specific version with `VERSION`:

```sh
curl -fsSL https://raw.githubusercontent.com/mivnixv/yamlext/main/install.sh | VERSION=v1.0.0 sh
```

### Manual download

Download a pre-built binary from the [Releases](https://github.com/mivnixv/yamlext/releases) page:

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `yamlext-linux-x86_64` |
| Linux aarch64 | `yamlext-linux-aarch64` |
| macOS x86_64 | `yamlext-macos-x86_64` |
| macOS aarch64 (Apple Silicon) | `yamlext-macos-aarch64` |
| Windows x86_64 | `yamlext-windows-x86_64.exe` |

Each release also includes a `checksums.txt` for verification:

```sh
sha256sum -c checksums.txt
```

### Build from source

```sh
cargo build --release
# binary at target/release/yamlext
```

## Releasing a new version

Tag a commit with a semver version to trigger the release pipeline:

```sh
git tag v1.2.3
git push origin v1.2.3
```

GitHub Actions will build all platform binaries and publish them as a GitHub Release automatically.

## Usage

```sh
yamlext input.yaml              # process and print to stdout
yamlext input.yaml > out.yaml   # redirect output to file
cat input.yaml | yamlext -      # read from stdin
```

## Custom Tags

### `!include`

Include the contents of another YAML file.

```yaml
# Include entire file
config: !include path/to/file.yaml

# Include a specific nested field (slash-separated path)
city: !include [path/to/file.yaml, "address/city"]

# Sequence indices are supported in the field path
first: !include [path/to/file.yaml, "items/0"]
```

Paths are relative to the file that contains the `!include` tag, so included files can use their own relative includes.

### `!merge`

Merge multiple YAML files into one. All files must be the same collection type.

**Mappings** — deep-merged left-to-right; later files override earlier ones:

```yaml
# As a value
merged: !merge [base.yaml, overrides.yaml, extras.yaml]

# At the root level — the whole document is the merge result
!merge [base.yaml, overrides.yaml, extras.yaml]
```

**Sequences** — concatenated left-to-right:

```yaml
# As a value
all_items: !merge [list1.yaml, list2.yaml]

# At the root level
!merge [list1.yaml, list2.yaml]
```

Merging a mapping with a sequence is an error.

## Examples

See the [`examples/`](examples/) directory:

| File | Description |
|------|-------------|
| [main.yaml](examples/main.yaml) | All tags used as nested values |
| [merge_mappings.yaml](examples/merge_mappings.yaml) | Root-level merge of mappings |
| [merge_sequences.yaml](examples/merge_sequences.yaml) | Root-level merge of sequences |
