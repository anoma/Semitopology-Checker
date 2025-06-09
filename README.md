# Semiframes and Semitopologies Checker

A Rust tool for finding semiframes and semitopologies using nauty for canonical graph labeling.

## Overview

This program finds all distinguished semiframes (union-closed covers that don't contain the empty set and distinguish all their points) or semitopologies up to isomorphism for a given size n using a depth-first search with batching and caching.

Actual model checking will be for a future release.

## Features

- **Semiframes Search**: Find all distinguished semiframes where every element is distinguished
- **Semitopologies Search**: Find all valid semitopologies (union-closed families)
- **Canonicalization**: Canonicalize individual semitopologies to their standard form
- **Canonical Graph Labeling**: Uses nauty library for efficient isomorphism checking
- **Batched Processing**: Memory-efficient processing with configurable batch sizes
- **Caching**: Configurable canonicalization cache for performance optimization
- **Range Support**: Search single sizes or ranges of sizes
- **Custom Starting Points**: Specify custom starting families
- **Output Control**: Configurable output files and generation limits

## Installation

This project requires the nauty library to be installed on your system. Build with:

```bash
cargo build --release
```

## Usage

This tool has two main commands:

- **`search`**: Find semiframes or semitopologies systematically
- **`canon`**: Canonicalize individual semitopologies to standard form

Use `cargo run -- <command> --help` for detailed help on each command.

### Search Command

```bash
# Search for semiframes of size 3
cargo run -- search -s 3

# Search for semiframes of sizes 1 through 5
cargo run -- search -s 1-5

# Search for semitopologies instead of semiframes
cargo run -- search -s 3 --semitopologies
```

### Canon Command

```bash
# Canonicalize a semitopology (auto-infers n=3)
cargo run -- canon -f "{{1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}"

# Canonicalize with explicit size
cargo run -- canon -f "{{1, 2}}" -n 4

# Example with asymmetric family
cargo run -- canon -f "{{3}, {1, 3}, {2, 3}, {1, 2, 3}}"
```

### Search Command Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--size` | `-s` | Size to search (number or range like "3-5") | `1-6` |
| `--cache-size` | `-c` | Maximum cache size (0 to disable) | `10000` |
| `--limit` | `-l` | Hard limit on families to generate (0 for unlimited) | `0` |
| `--output` | `-o` | Output file pattern (use `{n}` for size placeholder) | `distinguished_families_n{n}.txt` |
| `--semitopologies` | | Search for semitopologies instead of semiframes | `false` |
| `--starting-family` | | Starting family as semitopology (e.g., "{{1}, {1,2}}") | `{{1,2,...,n}}` |
| `--batch-size` | `-b` | Batch size for processing | `100000` |
| `--log-interval` | | Log interval for progress reporting | `10000` |

### Canon Command Options

| Option | Short | Description | Required |
|--------|-------|-------------|----------|
| `--family` | `-f` | The semitopology to canonicalize (e.g., "{{1, 2}, {1, 3}}") | Yes |
| `--size` | `-n` | Size n for the semitopology (auto-inferred if not provided) | No |

### Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help message |
| `--version` | `-V` | Show version information |

### Example Commands

#### Basic Searches

```bash
# Find semiframes of size 4
cargo run -- search -s 4

# Find first 10 semiframes of size 5
cargo run -- search -s 5 -l 10

# Find all semitopologies of size 3
cargo run -- search -s 3 --semitopologies
```

#### Performance Tuning

```bash
# Disable caching for large searches
cargo run -- search -s 6 -c 0

# Use smaller batch size for memory-constrained systems
cargo run -- search -s 5 -b 1000

# Increase log frequency for more progress updates
cargo run -- search -s 5 --log-interval 1000
```

#### Custom Output and Starting Points

```bash
# Custom output file pattern
cargo run -- search -s 3 -o "results_size_{n}.txt"

# Start from a specific family instead of {{1,2,...,n}}
# Note: starting family is automatically canonicalized
cargo run -- search -s 4 --starting-family "{{1}, {2}, {1,2}}"

# Asymmetric input gets canonicalized before search
cargo run -- search -s 3 --starting-family "{{3}, {1,3}, {2,3}}"
# Uses canonicalized form: {{1}, {1,2}, {1,3}}

# Search semitopologies with custom output
cargo run -- search -s 3 --semitopologies -o "semitopologies_{n}.txt"
```

#### Canonicalization Examples

```bash
# Basic canonicalization (size auto-inferred)
cargo run -- canon -f "{{2, 3}, {1, 3}, {1, 2}, {1, 2, 3}}"

# Canonicalize asymmetric family
cargo run -- canon -f "{{3}, {1, 3}, {2, 3}, {1, 2, 3}}"
# Output: {{1}, {1, 2}, {1, 3}, {1, 2, 3}}

# Specify size explicitly
cargo run -- canon -f "{{1}, {2}}" -n 5

# Empty family
cargo run -- canon -f "{}" -n 3
```

## Expected Outputs

### Semiframes vs Semitopologies

**Semiframes** (default): Only families where every element is distinguished by some set in the family.

**Semitopologies** (with `--semitopologies`): All union-closed families, regardless of distinguishing property.

### Sample Outputs

For `n=2` semiframes:
```
{{1}, {1, 2}}
{{1}, {2}, {1, 2}}
```

For `n=3` with limit 3:
```
{{1}, {2}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}
{{1}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}
{{2}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}
```

For `n=3` semitopologies (first 5):
```
{{1, 2, 3}}
{{1}, {1, 2, 3}}
{{1, 2}, {1, 2, 3}}
{{1}, {1, 2}, {1, 2, 3}}
{{2}, {3}, {2, 3}, {1, 2, 3}}
```

### Performance Characteristics

| Size | Actual Semiframes Count | Time (measured) | Memory Usage |
|------|------------------------|-----------------|--------------|
| n=1  | 1                      | <0.01s          | Trivial      |
| n=2  | 2                      | <0.01s          | Trivial      |
| n=3  | 10                     | <0.01s          | Trivia       |
| n=4  | 138                    | <0.01s          | Low          |
| n=5  | 14,005                 | ~0.43s          | Medium       |
| n=6  | >5,000,000             | >1 hour         | High (may run out of memory) |
| n=7  | Hundreds of billions?  | ???             | Very High (hope you have a lot of ram) |

*Performance measured on the current implementation. Times may vary by system.*

## Algorithm Details

1. **Canonical Search**: Uses nauty library to ensure each family is found exactly once up to isomorphism
2. **Depth-First Exploration**: Systematically explores all possible extensions of families
3. **Union-Closure**: Only considers families that are closed under union operations
4. **Batched Processing**: Processes families in batches to manage memory usage
5. **Caching**: Caches canonical forms to avoid redundant computations

## Output Format

Results are saved to text files with one family per line, formatted as sets of sets:
```
{{1}, {2}, {1, 2}}
{{1, 3}, {2, 3}, {1, 2, 3}}
```

Each line represents one distinct family up to isomorphism.

## Performance Tips

- **Large searches**: Use `--cache-size 0` to disable caching and save memory
- **Memory constraints**: Reduce `--batch-size` to process smaller batches
- **Time limits**: Use `--limit` to cap the number of results
- **Progress monitoring**: Decrease `--log-interval` for more frequent updates

## Troubleshooting

- If memory usage is high, try `--cache-size 0` or smaller `--batch-size`
- For very large searches, consider using `--limit` to get partial results
- Starting family elements must be valid for the given size n (1 ≤ element ≤ n)
