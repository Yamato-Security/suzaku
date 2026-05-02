# GitHub Copilot Instructions for Suzaku

## Project Overview

**Suzaku** is a cloud log threat detection and fast forensics tool developed by Yamato Security, designed for AWS CloudTrail and Azure logs.  
It analyzes logs using the Sigma rule engine (`sigma-rust`) and outputs timelines in CSV/JSON/JSONL format.

- **Language**: Rust (edition = "2024", rust-version = "1.89.0")
- **Version**: `2.0.0-dev`
- **Memory Allocator**: `mimalloc` (configured as global allocator)
- **Parallelism**: `rayon`

---

## Directory Structure

```
suzaku/
├── Cargo.toml
├── config/
│   ├── aws_profile.yaml      # AWS CloudTrail output profile definition
│   └── azure_profile.yaml    # Azure output profile definition
├── rules/
│   ├── sigma/                # Sigma rules (AWS / Azure)
│   └── suzaku/               # Suzaku-specific rules
├── art/
│   └── logo.txt              # ASCII art displayed at startup
└── src/
    ├── main.rs               # Entry point & subcommand routing
    ├── cmd/                  # Subcommand implementations
    │   ├── update.rs         # update-rules command (updates rules repo via git2)
    │   ├── aws/
    │   │   ├── aws_timeline.rs   # aws-ct-timeline command
    │   │   ├── aws_search.rs     # aws-ct-search command
    │   │   ├── aws_metrics.rs    # aws-ct-metrics command
    │   │   └── aws_summary.rs    # aws-ct-summary command
    │   └── azure/
    │       └── azure_timeline.rs # azure-timeline command
    ├── core/                 # Core logic
    │   ├── color.rs          # SuzakuColor enum & terminal colors
    │   ├── log_source.rs     # LogSource enum (Aws / Azure / All)
    │   ├── rules.rs          # Sigma rule loading & filtering
    │   ├── scan.rs           # File/directory scanning
    │   ├── summary.rs        # DetectionSummary & detection summary display
    │   ├── timeline.rs       # make_timeline() main processing
    │   ├── timeline_writer.rs # OutputContext & output control
    │   └── util.rs           # Utility functions
    └── option/               # CLI option definitions
        ├── cli.rs            # clap subcommand definitions
        ├── geoip.rs          # MaxMind GeoIP lookup
        └── timefiler.rs      # Time filtering
```

---

## Subcommand List

| Command | Description |
|---|---|
| `aws-ct-timeline` | Generate a DFIR timeline from AWS CloudTrail logs |
| `aws-ct-search` | Search AWS CloudTrail logs by keyword/regex |
| `aws-ct-metrics` | Generate per-field metrics from AWS CloudTrail logs |
| `aws-ct-summary` | Generate a summary from AWS CloudTrail logs |
| `azure-timeline` | Generate a DFIR timeline from Azure logs |
| `update-rules` | Update the rules repository via git2 |

---

## Key Types & Structs

### `LogSource` (`src/core/log_source.rs`)
An enum representing the log source type.
```rust
pub enum LogSource {
    Aws,   // CloudTrail: profile = config/aws_profile.yaml
    Azure, // Activity/Audit/SignIn Logs: profile = config/azure_profile.yaml
    All,
}
```
- `profile_path()` → Returns the corresponding YAML profile path
- `supported_services()` → Returns a slice of service names that match `logsource.service` in Sigma rules

### `SuzakuColor` (`src/core/color.rs`)
An enum for terminal color output.
```rust
pub enum SuzakuColor { Red, Orange, Green, Yellow, Cyan, White }
```
- `rdg(no_color: bool) -> Option<Color>`: Returns `None` when `no_color=true` (disables color)

### `OutputConfig` / `OutputContext` (`src/core/timeline_writer.rs`)
The central structs for output control.
- `OutputConfig { no_color, raw_output }` — configuration values
- `OutputContext<'a>` — holds the profile, GeoIP, writers, and write state

### `DetectionSummary` (`src/core/summary.rs`)
Aggregated data from scan results.
```rust
pub struct DetectionSummary {
    pub author_titles: HashMap<String, HashSet<String>>,
    pub timestamps: Vec<i64>,
    pub total_events: usize,
    pub event_with_hits: usize,
    pub dates_with_hits: HashMap<String, HashMap<String, usize>>,
    pub level_with_hits: HashMap<String, HashMap<String, usize>>,
    pub first_event_time: Option<DateTime<Utc>>,
    pub last_event_time: Option<DateTime<Utc>>,
}
```

### CLI Option Structs (`src/option/cli.rs`)
| Struct | Purpose |
|---|---|
| `CommonOptions` | Shared across all commands (`--no-color`, `--quiet`, `--debug`) |
| `InputOption` | Input file/directory + time filters |
| `OutputOption` | Output destination, output type (1-5), thread count, GeoIP, etc. |
| `TimelineOptions` | Timeline-specific (rules path, minimum level, no-summary) |
| `SearchOptions` | Search-specific (keywords, regex, field filters) |
| `TimeOption` | `--timeline-start` / `--timeline-end` / `--time-offset` |
| `FileDateOption` | `--file-date-from` / `--file-date-to` (S3 path date filter) |

---

## Coding Conventions & Patterns

### 1. Use `p()` for all terminal output
```rust
// src/core/util.rs
pub fn p(color: Option<Color>, msg: &str, newline: bool)
```
- **Always** use the `p()` function from `termcolor`; avoid `println!` / `print!` except for the startup banner
- Use the `SuzakuColor::Green.rdg(no_color)` pattern to respect the `no_color` flag

```rust
// Good
p(Green.rdg(no_color), "Detections: ", false);
p(None, &count.to_string(), true);

// Bad (no_color flag is ignored)
println!("\x1b[32mDetections: {count}\x1b[0m");
```

### 2. Steps to add a new subcommand
1. Add a new variant to the `Commands` enum in `src/option/cli.rs` (with `#[command(...)]` attribute)
2. Create the corresponding module file under `src/cmd/`
3. Add the new pattern to `match cmd { ... }` in `src/main.rs`
4. Add the module declaration to `src/cmd.rs` or `src/cmd/mod.rs`

### 3. Output format numbers
| `output_type` | Format |
|---|---|
| `1` | CSV (default) |
| `2` | JSON |
| `3` | JSONL |
| `4` | CSV + JSON |
| `5` | CSV + JSONL |

`--raw-output` is only valid for JSON-based formats (2-5). It cannot be used with CSV (1).

### 4. Rule level hierarchy
`informational (1) < low (2) < medium (3) < high (4) < critical (5)`

`filter_rules_by_level()` excludes rules below the `--min-level` threshold.  
Valid `--min-level` values: `informational`, `info`, `low`, `medium`, `med`, `high`, `critical`, `crit`

### 5. Profile format (`config/*.yaml`)
YAML format (actually parsed line-by-line as `key: 'value'`).
- `.fieldName` → references a top-level JSON field
- `.nested.field` → references a nested JSON field
- `sigma.title` / `sigma.author` / `sigma.level` / `sigma.id` → metadata from the matched Sigma rule
- `|` separator allows fallback across multiple fields (e.g. `.time|.eventTimestamp`)
- If a `SrcIP` field exists and GeoIP is enabled, `SrcASN`, `SrcCity`, `SrcCountry` are added automatically

### 6. Scan processing flow
```
make_timeline()
  └─ rules::load_rules_from_dir()      # Load Sigma rules
  └─ rules::filter_rules_by_level()    # Filter by level
  └─ scan_directory() / scan_file()    # Read & scan log files
      └─ detect_events()               # Match Sigma rules against events
          └─ write_record()            # Output via OutputContext
  └─ print_summary()                   # Display results summary
```

### 7. Time filtering (`src/option/timefiler.rs`)
- `filter_by_time(opt, value, ts_key)`: Filter by RFC 3339 timestamp field
- `filter_file_by_date_path(opt, path)`: Filter files by S3-compatible `YYYY/MM/DD` path structure
- `--time-offset` format: `1y` (1 year), `3M` (3 months), `30d` (30 days), `24h` (24 hours), `30m` (30 minutes)

### 8. GeoIP (`src/option/geoip.rs`)
Three MaxMind `.mmdb` files are required:
- `GeoLite2-ASN.mmdb`
- `GeoLite2-Country.mmdb`
- `GeoLite2-City.mmdb`

Specify the directory containing these files with `--geo-ip <DIR>`.

---

## Key Dependencies

| Crate | Purpose |
|---|---|
| `clap` 4.5 (derive) | CLI parsing |
| `sigma-rust` (Yamato-Security fork) | Sigma rule engine |
| `serde_json` | JSON parsing |
| `rayon` | Parallel iterators |
| `csv` | CSV read/write |
| `chrono` | Date/time handling |
| `flate2` (zlib-rs) | gzip decompression |
| `maxminddb` | GeoIP lookup |
| `git2` | Rule updates (git clone/pull) |
| `comfy-table` | Terminal table output |
| `indicatif` | Progress bars |
| `termcolor` | Color terminal output |
| `mimalloc` | High-performance memory allocator |
| `regex` | Regular expressions (for aws-ct-search) |
| `cidr-utils` | IP CIDR matching (for GeoIP) |
| `hashbrown` | High-performance HashMap/HashSet |

---

## Testing & Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

Test log files are stored in the `test_files/` directory (JSON/gzip format).

---

## Important Notes

- Actively uses Rust edition 2024 features such as `let-chain`
- `#[allow(clippy::too_many_arguments)]` is applied to functions with many arguments such as `scan_file` / `scan_directory`
- The `no_color` flag is obtained from the subcommand's `common_opt.no_color` and must be propagated to all downstream functions
- Existing output files will not be overwritten without the `--clobber (-C)` option (checked upfront in `main.rs`)
- Thread count for parallel processing is set via `set_rayon_threat_number()` (0 = auto-set to number of CPU cores)
- Azure logs support both `graph API format` (`value` key array) and `activitylogs format`
