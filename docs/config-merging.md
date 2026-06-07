# Spec: Config Merging with Layered Overrides

## Goal

Replace the single-file config loading with a **layered merge pipeline**: hardcoded defaults → system config → user config → CLI arguments. Gives users flexibility in where they define settings and how those settings compose.

## Scope

### In scope
- Add `anyhow` and `dirs` dependencies to `Cargo.toml`
- `Default` implementations for all config structs (`TomlConfig`, `TomlBucketConfig`, `TomlLogConfig`, `Balde`)
- Override structs (all fields `Option<T>`) for each TOML-loadable config struct:
  - `TomlConfigOverride` (wraps `bucket`, `log`, `baldes`)
  - `TomlBucketConfigOverride`
  - `TomlLogConfigOverride`
  - `BaldeOverride`
- `merge()` method on each final config struct to apply overrides in place
- System config path: `/etc/balde/config.toml`
- User config path: `~/.config/balde/balde.toml` (overridable via `--balde-config-file` CLI arg)
- `load_merged_config()` function that executes the full pipeline: defaults → system TOML → user TOML → CLI
- `main.rs` wired to use `load_merged_config()` instead of `load_config()`
- `baldes` overrides replace the entire HashMap (no per-entry merging for now)

### Out of scope
- Environment variable overrides
- Per-baldes entry merging (whole HashMap is replaced)
- Validation beyond what `Deserialize` already does
- Macro-based merge generation (explicit `merge()` methods are fine for the current ~10 fields)
- Adding new CLI flags for bucket fields (CLI only overrides `log_level` and `log_path`, plus `balde_config_file` for the user config path)

## Design

### Approach

Manual merge pattern (no `config` crate). Two parallel type trees:

- **Final configs** — all fields concrete, implement `Default`
- **Override structs** — all fields `Option<T>`, implement `Default` (all `None`), derive `Deserialize` for TOML loading

Each final config gets a `merge(&mut self, overrides: &XOverride)` method. The pipeline is `default → merge(system) → merge(user) → merge(cli)`.

### Merge priority (lowest to highest)

1. Hardcoded defaults (`Default::default()`)
2. System config (`/etc/balde/config.toml`)
3. User config (`~/.config/balde/balde.toml` or `--balde-config-file`)
4. CLI arguments (`--log-level`, `--log-path`)

### Components / Files

| File | Action | What |
|---|---|---|
| `Cargo.toml` | Modify | Add `anyhow` and `dirs` dependencies |
| `src/config.rs` | Rewrite | Final structs + `Default` + override structs + `merge()` methods + `load_toml_file()` + `load_merged_config()` |
| `src/cli.rs` | Modify | Keep `balde_config_file`, `log_level`, `log_path`. Add `From<CliConfig> for TomlLogConfigOverride` |
| `src/main.rs` | Modify | Replace `load_config()` call with `load_merged_config()` |
| `src/log.rs` | No change | `LogLevel` stays as-is, referenced from config |
| `src/files.rs` | No change | Stub, unchanged |

### Data flow

```
TomlConfig::default()
  └─→ merge(system overrides from /etc/balde/config.toml)
        └─→ merge(user overrides from ~/.config/balde/balde.toml or --balde-config-file)
              └─→ merge(log overrides from CLI --log-level / --log-path)
                    └─→ final TomlConfig
```

### Type definitions (key structs)

```rust
// ── Final configs ──

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    pub bucket: TomlBucketConfig,
    pub baldes: HashMap<String, Balde>,
    pub log: TomlLogConfig,
}

#[derive(Debug, Deserialize)]
pub struct TomlBucketConfig {
    pub name: String,
    pub region: String,
    pub url: String,
    pub key_id: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct TomlLogConfig {
    pub path: PathBuf,
    pub level: LogLevel,
}

#[derive(Debug, Deserialize)]
pub struct Balde {
    pub name: String,
    #[serde(default)]
    pub filter: Vec<String>,
    pub path: PathBuf,
}

// ── Override structs (Deserialize for TOML, Default for all-None) ──

#[derive(Debug, Default, Deserialize)]
pub struct TomlConfigOverride {
    pub bucket: Option<TomlBucketConfigOverride>,
    pub baldes: Option<HashMap<String, BaldeOverride>>,
    pub log: Option<TomlLogConfigOverride>,
}

#[derive(Debug, Default, Deserialize)]
pub struct TomlBucketConfigOverride {
    pub name: Option<String>,
    pub region: Option<String>,
    pub url: Option<String>,
    pub key_id: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct TomlLogConfigOverride {
    pub path: Option<PathBuf>,
    pub level: Option<LogLevel>,
}

#[derive(Debug, Default, Deserialize)]
pub struct BaldeOverride {
    pub name: Option<String>,
    #[serde(default)]
    pub filter: Option<Vec<String>>,
    pub path: Option<PathBuf>,
}
```

### Merge methods

```rust
impl TomlConfig {
    pub fn merge(&mut self, overrides: &TomlConfigOverride) {
        if let Some(ref b) = overrides.bucket { self.bucket.merge(b); }
        if let Some(ref b) = overrides.baldes { self.baldes = b.clone(); }
        if let Some(ref l) = overrides.log { self.log.merge(l); }
    }
}

impl TomlBucketConfig {
    pub fn merge(&mut self, overrides: &TomlBucketConfigOverride) {
        if let Some(ref v) = overrides.name    { self.name = v.clone(); }
        if let Some(ref v) = overrides.region  { self.region = v.clone(); }
        if let Some(ref v) = overrides.url     { self.url = v.clone(); }
        if let Some(ref v) = overrides.key_id  { self.key_id = v.clone(); }
        if let Some(ref v) = overrides.key     { self.key = v.clone(); }
    }
}

impl TomlLogConfig {
    pub fn merge(&mut self, overrides: &TomlLogConfigOverride) {
        if let Some(ref v) = overrides.path  { self.path = v.clone(); }
        if let Some(ref v) = overrides.level { self.level = *v; }
    }
}

impl Balde {
    pub fn merge(&mut self, overrides: &BaldeOverride) {
        if let Some(ref v) = overrides.name   { self.name = v.clone(); }
        if let Some(ref v) = overrides.filter { self.filter = v.clone(); }
        if let Some(ref v) = overrides.path   { self.path = v.clone(); }
    }
}
```

### Loading function

```rust
fn load_toml_file<T: DeserializeOwned>(path: &PathBuf) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let config: T = toml::from_str(&contents)
        .with_context(|| format!("Invalid TOML in {}", path.display()))?;
    Ok(Some(config))
}

pub fn load_merged_config(
    user_config_path_override: Option<PathBuf>,
) -> Result<TomlConfig> {
    let mut config = TomlConfig::default();

    // Layer 1: System config
    let system_path = PathBuf::from("/etc/balde/config.toml");
    if let Some(overrides) = load_toml_file::<TomlConfigOverride>(&system_path)? {
        config.merge(&overrides);
    }

    // Layer 2: User config
    let user_path = match user_config_path_override {
        Some(p) => p,
        None => {
            let home = dirs::home_dir()
                .context("Could not determine home directory")?;
            home.join(".config/balde/balde.toml")
        }
    };
    if let Some(overrides) = load_toml_file::<TomlConfigOverride>(&user_path)? {
        config.merge(&overrides);
    }

    // Layer 3: CLI overrides (only log fields)
    let cli = CliConfig::parse();
    let log_overrides: TomlLogConfigOverride = cli.into();
    config.log.merge(&log_overrides);

    Ok(config)
}
```

### CLI struct

```rust
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct CliConfig {
    /// Path to the user config file (overrides ~/.config/balde/balde.toml)
    #[arg(long)]
    pub balde_config_file: Option<PathBuf>,

    /// Override the log level
    #[arg(long)]
    pub log_level: Option<LogLevel>,

    /// Override the log output path
    #[arg(long)]
    pub log_path: Option<PathBuf>,
}

impl From<CliConfig> for TomlLogConfigOverride {
    fn from(cli: CliConfig) -> Self {
        TomlLogConfigOverride {
            level: cli.log_level,
            path: cli.log_path,
        }
    }
}
```

### main.rs changes

```rust
fn main() {
    let cli_config = CliConfig::parse();
    let config = match config::load_merged_config(cli_config.balde_config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {:#}", e);
            exit(127);
        }
    };
    println!("Config loaded: {:#?}", config);
    println!("Baldeee");
}
```

Perf note: `CliConfig::parse()` is called twice (once in `main` for `balde_config_file`, once inside `load_merged_config` for the overrides). This is fine — `clap::Parser::parse()` is idempotent and fast. Alternative: pass the full `CliConfig` into `load_merged_config` to avoid the double parse. Do whichever reads cleaner.

## Edge Cases & Error Handling

| Scenario | Handling |
|---|---|
| System config file missing (`/etc/balde/config.toml` doesn't exist) | Silently skip (`load_toml_file` returns `Ok(None)`) |
| User config file missing | Silently skip |
| Config file exists but is invalid TOML | `anyhow`-wrapped error: "Invalid TOML in /path/to/file" |
| CLI provides `--balde-config-file` | Use that path instead of `~/.config/balde/balde.toml` |
| `dirs::home_dir()` returns `None` | `anyhow::Context` error: "Could not determine home directory" |
| `baldes` override is `Some(map)` | Replace entire HashMap; defaults are discarded for that field |
| `baldes` override is `None` | Keep the in-memory value from the previous layer |
| No CLI args | All CLI overrides are `None`; final config = defaults + system + user |
| `canonicalize` fails on a config path | Propagate the IO error via `anyhow` |

## Verification

1. **Unit test in config.rs:** Create a `ConfigOverride` with one field set, merge into defaults, assert only that field changed.
   ```bash
   cargo test config::tests::merge_overrides
   ```
2. **Manual test — defaults only:** With no system or user config files present, run:
   ```bash
   cargo run
   ```
   Expected: logs show all default values.
3. **Manual test — system config partial override:**
   ```bash
   # Create /etc/balde/config.toml with only [log] level = "Trace"
   cargo run
   ```
   Expected: log.level = Trace, everything else from defaults.
4. **Manual test — CLI wins over all:**
   ```bash
   cargo run -- --log-level Fatal
   ```
   Expected: log.level = Fatal regardless of file contents.
5. **Manual test — invalid TOML:**
   ```bash
   # Write invalid TOML to ~/.config/balde/balde.toml
   cargo run
   ```
   Expected: clear error message with file path, exit code 127.
6. **Manual test — alternate user config:**
   ```bash
   cargo run -- --balde-config-file /tmp/custom.toml
   ```
   Expected: loads `/tmp/custom.toml` instead of the default user path.
