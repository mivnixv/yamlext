# yamlext

A CLI tool that extends YAML with `!include` and `!merge` custom tags.

## Install

```sh
cargo build --release
# binary at target/release/yamlext
```

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
