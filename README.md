# uyamlt
`uyamlt` (**U**nity**YAML**Merge **T**rampoline; pronounced `U-yaml-T`) is a CLI tool simplifies the usage of Unity's Smart Merge Tool (`UnityYAMLMerge`) for systems with multiple Unity installations. `uyamlt` automatically finds the most appropriate version of `UnityYAMLMerge` (falling back to the latest version if it isn't in a project) and executes it.

## Installing

If you are a rust developer, run the following from your favourite shell:
```shell
cargo install uyamlt
```

## Building

To install `uyamlt`, you need Rust and Cargo installed on your system. Once you have Rust set up, run the following command:

```shell
git clone https://github.com/shloon/uyamlt
cd uyamlt
cargo build --release
UYAMLT_DRY_RUN=1 ./target/release/uyamlt
```

## Configuring git
Once you have `uyamlt` in your path, you can add this to your .git/config file:
```
[merge]
    tool = unityyamlmerge
    
[mergetool "unityyamlmerge"]
	trustExitCode = false
	keepTemporaries = true
	keepBackup = false
	cmd = uyamlt merge -p "$BASE" "$REMOTE" "$LOCAL" "$MERGED"

[merge "unityyamlmerge"]
     name = UnityYAMLMerge
     driver = uyamlt merge --force --fallback none %O %B %A %P
```

This will set up `unityyamlmerge` as the default merge tool, and provide a merge strategy for `.gitattributes` files.

## Current Limitations
- `uyamlt` currently only supports editors that were **installed** by UnityHub (Editors added manually are unsupported).
- `uyamlt` does NOT support any third-party UnityHub alternative.
- `uyamlt` does NOT manage the "mergerules.txt" file, thus you'll have to manually find and edit the correct file for the mergetool txt in question.
