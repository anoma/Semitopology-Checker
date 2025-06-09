# Semiframes and Semitopologies Checker

A Rust tool for finding semiframes and semitopologies using nauty for canonical graph labeling, with model checking capabilities for proposition logic.

## Overview

This program finds all distinguished semiframes (union-closed covers that don't contain the empty set and distinguish all their points) or semitopologies up to isomorphism for a given size n using a depth-first search with batching and caching.

Additionally, it provides model checking functionality to verify semitopologies against logical propositions and search for semitopologies satisfying specific formulas.

## Features

- **Semiframes Search**: Find all distinguished semiframes where every element is distinguished
- **Semitopologies Search**: Find all valid semitopologies (union-closed families)
- **Model Checking**: Verify if semitopologies satisfy logical propositions
- **Formula Search**: Find semitopologies that satisfy specific logical formulas
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

This tool has four main commands:

- **`search`**: Find semiframes or semitopologies systematically
- **`canon`**: Canonicalize individual semitopologies to standard form
- **`check`**: Check if a semitopology satisfies a given logical formula
- **`find`**: Find semitopologies that satisfy a given logical formula

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

### Check Command

```bash
# Check if a semitopology satisfies a simple formula
cargo run -- check -f "EO X. EP x. x in X" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Check a complex formula with universal and existential quantifiers
cargo run -- check -f "AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)" -s "{{}, {1}, {2}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}" -n 3

# Check nonempty property
cargo run -- check -f "EO X. nonempty X" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Check with auto-inferred size
cargo run -- check -f "EO X. EP x. x in X" -s "{{}, {1, 2, 3}}"
```

### Find Command

```bash
# Find one semitopology satisfying a formula (default)
cargo run -- find -f "EO X. EP x. x in X" -s 3 --semitopologies

# Find multiple semitopologies satisfying a formula
cargo run -- find -f "EO X. EP x. x in X" -s 3 -l 5 --semitopologies

# Save results to file instead of console output
cargo run -- find -f "AO X. x in X" -s 3 -o "results_n{n}.txt" --semitopologies

# Search for semiframes satisfying a formula
cargo run -- find -f "EP x. AO X. x in X" -s 4 -l 10
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

### Check Command Options

| Option | Short | Description | Required |
|--------|-------|-------------|----------|
| `--formula` | `-f` | The logical formula to check (e.g., "EO X. EP x. x in X") | Yes |
| `--semitopology` | `-s` | The semitopology to check against (e.g., "{{1, 2}, {1, 3}}") | Yes |
| `--size` | `-n` | Size n for the semitopology (auto-inferred if not provided) | No |

### Find Command Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--formula` | `-f` | The logical formula to satisfy (e.g., "EO X. EP x. x in X") | Required |
| `--size` | `-s` | Size to search (number or range like "3-5") | `1-6` |
| `--limit` | `-l` | Maximum number of results to find | `1` |
| `--output` | `-o` | Output file pattern (optional, use {n} for size placeholder) | Console output |
| `--cache-size` | `-c` | Maximum cache size (0 to disable) | `10000` |
| `--semitopologies` | | Search for semitopologies instead of semiframes | `false` |
| `--starting-family` | | Starting family as semitopology | `{{1,2,...,n}}` |
| `--batch-size` | `-b` | Batch size for processing | `100000` |
| `--log-interval` | | Log interval for progress reporting | `10000` |

### Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help message |
| `--version` | `-V` | Show version information |

## Proposition Language

The model checker supports a rich proposition language for describing properties of semitopologies.

### Syntax

**Variables:**
- Point variables: lowercase letters (e.g., `x`, `y`, `p`)
- Open variables: uppercase letters (e.g., `X`, `Y`, `U`)

**Logical Operators:**
- `&&`: Logical AND
- `||`: Logical OR  
- `=>`: Logical implication
- `!`: Logical negation
- `()`: Parentheses for grouping

**Quantifiers:**
- `AP x.`: Universal quantification over points (for all points x)
- `EP x.`: Existential quantification over points (there exists a point x)
- `AO X.`: Universal quantification over opens (for all opens X)
- `EO X.`: Existential quantification over opens (there exists an open X)

**Primitive Relations:**
- `x in X`: Point x is in open X
- `X inter Y`: Open X intersects open Y (their intersection is non-empty)
- `nonempty X`: Open X is nonempty (contains at least one point)

### Examples

```
# Simple membership
x in X

# Intersection
X inter Y

# Nonempty property
nonempty X

# Existential statements
EO X. EP x. x in X
# "There exists an open X such that there exists a point x such that x is in X"

# Universal statements  
AO X. AP x. x in X => (x in Y)
# "For all opens X, for all points x, if x is in X then x is in Y"

# Nonempty properties
EO X. nonempty X
# "There exists a nonempty open X"

# Complex formula with mixed quantifiers
AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)
# "For all opens X, there exists an open Y such that for all points x,
#  if either x is in X or X intersects Y, then x is not in Y"

# Conjunction and disjunction
(x in X) && (y in Y) || (X inter Y)

# Implication chains
(x in X) => (X inter Y) => (y in Z)
```

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

#### Model Checking Examples

```bash
# Check simple existence property
cargo run -- check -f "EO X. EP x. x in X" -s "{{1, 2}, {1, 3}}" -n 3

# Check that a semitopology satisfies a universal property
cargo run -- check -f "AO X. AP x. x in X => (X inter Y)" -s "{{1}, {2}, {1, 2}}" -n 2

# Complex formula verification
cargo run -- check -f "AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)" \
  -s "{{1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}" -n 3

# Find semitopologies with specific properties (console output)
cargo run -- find -f "EO X. EP x. x in X" -s 3 --semitopologies

# Find multiple results
cargo run -- find -f "AO X. AP x. x in X" -s 3 -l 5 --semitopologies

# Save model checking results to file
cargo run -- find -f "EP x. EO X. x in X" -s 3 -l 10 \
  -o "satisfying_semitopologies_n{n}.txt" --semitopologies

# Search for semiframes satisfying a formula
cargo run -- find -f "AP x. EO X. x in X" -s 4 -l 3
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

### Model Checking Outputs

**Check command successful result:**
```bash
$ cargo run -- check -f "EO X. EP x. x in X" -s "{{1, 2}, {1, 3}, {1, 2, 3}}"
```
```
Formula: EO X. EP x. x in X
Semitopology (n=3): {{1, 2}, {1, 3}, {1, 2, 3}}
Result: ✓ SATISFIED
Witnesses:
  x = point 1
  X = {1, 2}
```

```bash
cargo run -- check -f "EO X. AP x. x in X" -s "{{1, 2}, {1, 3}, {1, 2, 3}}"
```
```
Formula: EP x. AO X. x in X
Semitopology (n=3): {{1, 2}, {1, 3}, {1, 2, 3}}
Result: ✓ SATISFIED
Witnesses:
  x = point 1
```

```bash
cargo run -- check -f "EP x. AO X. x in X" -s "{{1, 2}, {1, 3}, {1, 2, 3}}"
```
```
Formula: EO X. AP x. x in X
Semitopology (n=3): {{1, 2}, {1, 3}, {1, 2, 3}}
Result: ✓ SATISFIED
Witnesses:
  X = {1, 2, 3}
```

**Check command failed result:**
```bash
$ cargo run -- check -f "AO X. AP x. x in X" -s "{{1, 2}, {1, 3}, {1, 2, 3}}"
```
```
Formula: AO X. AP x. x in X
Semitopology (n=3): {{1, 2}, {1, 3}, {1, 2, 3}}
Result: ✗ NOT SATISFIED
```

**Find command console output:**
```bash
$ cargo run -- find -f "EO X. EP x. x in X" -s 3 -l 3 --semitopologies
```
```
Searching for semitopologies satisfying formula: EO X. EP x. x in X
--- Generating semitopologies satisfying formula for n=3 (Console Output) ---
  Starting family: {{1, 2, 3}}
{{1, 2, 3}}
{{1}, {1, 2, 3}}
{{1, 2}, {1, 2, 3}}

Results for n=3:
Total semitopologies satisfying formula: 3
```

```bash
$ cargo run -- find -f "EP x. AO X. x in X" -s 3 -l 10
```
```
Searching for semitopologies satisfying formula: EP x. AO X. x in X
--- Streaming semiframes satisfying formula for n=3 ---
  Log interval: 10000. Cache size: 10000. Limit: 10.
  Starting family: {{1, 2, 3}}

{{1}, {1, 2}, {1, 2, 3}}
{{1, 3}, {2, 3}, {1, 2, 3}}
{{1}, {1, 2}, {1, 3}, {1, 2, 3}}

  Search complete.
  Done.

Results for n=3:
Total semiframes satisfying formula: 3
```

### Performance Characteristics

| Size | Actual Semiframes Count | Time (measured) | Memory Usage |
|------|------------------------|-----------------|--------------|
| n=1  | 1                      | <0.01s          | Trivial      |
| n=2  | 2                      | <0.01s          | Trivial      |
| n=3  | 10                     | <0.01s          | Trivia       |
| n=4  | 138                    | <0.01s          | Low          |
| n=5  | 14,005                 | ~0.43s          | Medium       |
| n=6  | >34,000,000            | hours?          | High (may run out of memory) |
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
