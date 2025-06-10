# Semiframes and Semitopologies Checker

A model checker for discovering finite semitopologies and semiframes that satisfy first-order logical properties, with comprehensive search and verification capabilities.

## Overview

This program searches for finite semitopologies (union-closed covers containing the empty set) and finite semiframes (finite semitopologies with the T0 property) that satisfy logical properties expressed in a rich first-order language. 

The primary use case is finding structures with specific topological properties by expressing those properties as logical formulas. The tool also supports systematic enumeration of all structures up to isomorphism and verification of individual structures against given formulas.

## Features

- **Semiframes Search**: Find all semiframes (union-closed covers with T0 property)
- **Semitopologies Search**: Find all semitopologies (union-closed covers)
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
# Search for semitopologies of size 3 (default)
cargo run -- search -s 3

# Search for semitopologies of sizes 1 through 5
cargo run -- search -s 1-5

# Search for semiframes instead of semitopologies
cargo run -- search -s 3 --semiframes
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
cargo run -- find -f "EO X. EP x. x in X" -s 3

# Find multiple semitopologies satisfying a formula
cargo run -- find -f "EO X. EP x. x in X" -s 3 -l 5

# Save results to file instead of console output
cargo run -- find -f "AO X. x in X" -s 3 -o "results_n{n}.txt"

# Search for semiframes satisfying a formula
cargo run -- find -f "EP x. AO X. x in X" -s 4 -l 10 --semiframes
```

### Search Command Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--size` | `-s` | Size to search (number or range like "3-5") | `1-6` |
| `--cache-size` | `-c` | Maximum cache size (0 to disable) | `10000` |
| `--limit` | `-l` | Hard limit on families to generate (0 for unlimited) | `0` |
| `--output` | `-o` | Output file pattern (use `{n}` for size placeholder) | `distinguished_families_n{n}.txt` |
| `--semiframes` | | Search for semiframes instead of semitopologies | `false` |
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
| `--semiframes` | | Search for semiframes instead of semitopologies | `false` |
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
- `<=>`: Material equivalence (if and only if)
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
- `p != q`: Points p and q are not equal
- `X != Y`: Opens X and Y are not equal
- `p = q`: Points p and q are equal
- `X = Y`: Opens X and Y are equal

**Open Expressions:**
- `K p`: Community of point p
- `IC O`: Interior complement of open O (largest open disjoint from O)

### Built-in Definitions

The checker supports several built-in notations that expand to more complex formulas:

| Notation | Definition |
|----------|------------|
| `O inter P inter Q` | `O inter P && P inter Q` |
| `p inter q` | `AO O. AO P. (p in O && q in P) => O inter P` |
| `p inter q inter r` | `p inter q && q inter r` |
| `transitive T` | `AO O. AO P. (O inter T && T inter P) => O inter P` |
| `topen T` | `nonempty T && transitive T` |
| `regular p` | `topen (K p)` |
| `irregular p` | `!(regular p)` |
| `weakly_regular p` | `p in (K p)` |
| `quasiregular p` | `nonempty (K p)` |
| `indirectly_regular p` | `EP q. p inter q && regular q` |
| `hypertransitive p` | `AO O. AO Q. (AO P. p in P => O inter P inter Q) => O inter Q` |
| `unconflicted p` | `AP x. AP y. x inter p inter y => x inter y` |
| `conflicted p` | `!(unconflicted p)` |
| `conflicted_space` | `AP p. conflicted p` |
| `unconflicted_space` | `AP p. unconflicted p` |
| `regular_space` | `AP p. regular p` |
| `irregular_space` | `AP p. irregular p` |
| `weakly_regular_space` | `AP p. weakly_regular p` |
| `quasiregular_space` | `AP p. quasiregular p` |
| `indirectly_regular_space` | `AP p. indirectly_regular p` |
| `hypertransitive_space` | `AP p. hypertransitive p` |

These built-in notations automatically bind fresh variables to avoid variable capture, ensuring correct logical interpretation.

### Examples

```
# Simple membership
x in X

# Intersection
X inter Y

# Triple intersection (built-in)
O inter P inter Q

# Point intersection (built-in)
p inter q

# Nonempty property
nonempty X

# Point inequality
p != q

# Point equality
p = q

# Open inequality
X != Y

# Open equality  
X = Y

# Material equivalence
(p in X) <=> (q in Y)

# Built-in predicates
regular p
transitive T
unconflicted_space

# Existential statements
EO X. EP x. x in X
# "There exists an open X such that there exists a point x such that x is in X"

# Universal statements  
AO X. AP x. x in X => (x in Y)
# "For all opens X, for all points x, if x is in X then x is in Y"

# Nonempty properties
EO X. nonempty X
# "There exists a nonempty open X"

# Community construction
AP p. nonempty (K p)
# "For all points p, the community of p is nonempty"

# Point in community
EP p. EP q. q in K p
# "There exists a point p such that there exists a point q such that q is in the community of p"

# Interior complement
EO O. IC O
# "There exists an open O such that the interior complement of O is nonempty"

# Point in interior complement
EO O. EP x. x in IC O
# "There exists an open O and a point x such that x is in the interior complement of O"

# Complex formula with mixed quantifiers
AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)
# "For all opens X, there exists an open Y such that for all points x,
#  if either x is in X or X intersects Y, then x is not in Y"

# Conjunction and disjunction
(x in X) && (y in Y) || (X inter Y)

# Implication chains
(x in X) => (X inter Y) => (y in Z)

# Material equivalence (if and only if)
(x in X) <=> (y in Y)
# "x is in X if and only if y is in Y"

# Point distinctness
AP p. AP q. (p != q) => !(p inter q)
# "For all distinct points p and q, they do not intersect"

# Equivalence with quantifiers
EO X. (nonempty X) <=> (EP p. p in X)
# "There exists an open X such that X is nonempty if and only if there exists a point p in X"

# Open distinctness
AO X. AO Y. (X != Y) => !(X inter Y) || (EP p. (p in X) && !(p in Y))
# "For all distinct opens X and Y, either they don't intersect or there exists a point in one but not the other"

# Point self-equality
AP p. p = p
# "Every point is equal to itself"

# Open self-equality  
AO X. X = X
# "Every open is equal to itself"
```

### Example Commands

#### Basic Searches

```bash
# Find semitopologies of size 4 (default)
cargo run -- search -s 4

# Find first 10 semitopologies of size 5
cargo run -- search -s 5 -l 10

# Find all semiframes of size 3
cargo run -- search -s 3 --semiframes
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
cargo run -- search -s 3 --starting-family "{{3}, {1,3}, {2,3}, {1, 2, 3}}"
# Uses canonicalized form: {{1}, {1,2}, {1,3}}

# Search semiframes with custom output
cargo run -- search -s 3 --semiframes -o "semiframes_{n}.txt"
```

#### Canonicalization Examples

```bash
# Basic canonicalization (size auto-inferred)
cargo run -- canon -f "{{}, {2, 3}, {1, 3}, {1, 2}, {1, 2, 3}}"

# Canonicalize asymmetric family
cargo run -- canon -f "{{}, {3}, {1, 3}, {2, 3}, {1, 2, 3}}"
# Output: {{}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}

# Specify size explicitly
cargo run -- canon -f "{{}, {1}, {2}, {1, 2}}" -n 5

# Empty family
cargo run -- canon -f "{}" -n 3
```

#### Model Checking Examples

```bash
# Check simple existence property
cargo run -- check -f "EO X. EP x. x in X" -s "{{}, {1, 2}, {1, 3}, {1, 2, 3}}" -n 3

# Check that a semitopology satisfies a universal property
cargo run -- check -f "AO X. AP x. x in X => (X inter Y)" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Check community properties
cargo run -- check -f "AP p. nonempty (K p)" -s "{{}, {1, 2}, {1, 3}, {1, 2, 3}}" -n 3

# Check point in community
cargo run -- check -f "EP p. EP q. q in K p" -s "{{}, {1, 2}, {1, 3}, {1, 2, 3}}" -n 3

# Check interior complement properties
cargo run -- check -f "EO O. EP x. x in IC O" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Complex formula verification
cargo run -- check -f "AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)" \
  -s "{{}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}" -n 3

# Find semitopologies with specific properties (console output, default)
cargo run -- find -f "EO X. EP x. x in X" -s 3

# Find multiple results
cargo run -- find -f "AO X. AP x. x in X" -s 3 -l 5

# Save model checking results to file
cargo run -- find -f "EP x. EO X. x in X" -s 3 -l 10 \
  -o "satisfying_semitopologies_n{n}.txt"

# Search for semiframes satisfying a formula
cargo run -- find -f "AP x. EO X. x in X" -s 4 -l 3 --semiframes

# Check built-in predicates (quantify over points)
cargo run -- check -f "AP p. regular p" -s "{{}, {1}, {2}, {1,2}}" -n 2

# Check space properties
cargo run -- check -f "regular_space" -s "{{}, {1}, {2}, {1,2}}" -n 2

# Find spaces with specific properties
cargo run -- find -f "unconflicted_space" -s 3

# Check point inequality
cargo run -- check -f "EP p. EP q. p != q" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Check material equivalence
cargo run -- check -f "EO X. EO Y. EP p. (p in X) <=> (p in Y)" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Complex example
cargo run -- check -f "AP p. AP q. (p != q) <=> !(p inter q)" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Check open inequality
cargo run -- check -f "EO X. EO Y. X != Y" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Check point equality
cargo run -- check -f "EP p. EP q. p = q" -s "{{}, {1}, {2}, {1, 2}}" -n 2

# Check open equality
cargo run -- check -f "EO X. EO Y. X = Y" -s "{{}, {1}, {2}, {1, 2}}" -n 2
```

## Expected Outputs

### Semiframes vs Semitopologies

Both semiframes and semitopologies are union-closed covers containing the empty set.

**Semitopologies** (default): All union-closed covers containing the empty set.

**Semiframes** (with `--semiframes`): Union-closed covers containing the empty set with the additional T0 property - all points are topologically distinct (for any two distinct points, there exists an open set containing one but not the other). This cuts down the search space.

### Sample Outputs

For `n=2` semitopologies (default):
```
{{}, {1, 2}}
{{}, {1}, {1, 2}}
{{}, {1}, {2}, {1, 2}}
```

For `n=2` semiframes (with `--semiframes`):
```
{{}, {1}, {1, 2}}
{{}, {1}, {2}, {1, 2}}
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
cargo run -- check -f "EP x. AO X. nonempty X => x in X" -s "{{}, {1, 2}, {1, 3}, {1, 2, 3}}"
```
```
Formula: EO X. AP x. x in X
Semitopology (n=3): {{}, {1, 2}, {1, 3}, {1, 2, 3}}
Result: ✓ SATISFIED
Witnesses:
  X = {1, 2, 3}
```

**Check command failed result:**
```bash
$ cargo run -- check -f "AO X. AP x. x in X" -s "{{}, {1, 2}, {1, 3}, {1, 2, 3}}"
```
```
Formula: AO X. AP x. x in X
Semitopology (n=3): {{}, {1, 2}, {1, 3}, {1, 2, 3}}
Result: ✗ NOT SATISFIED
```

**Find command console output:**
```bash
$ cargo run -- find -f "EO X. EP x. x in X" -s 3 -l 3
```
```
Searching for semitopologies satisfying formula: EO X. EP x. x in X
--- Streaming semitopologies satisfying formula for n=3 ---
  Log interval: 10000. Cache size: 10000. Limit: 3.
  Starting family: {{}, {1, 2, 3}}

{{}, {1, 2, 3}}
{{}, {1}, {1, 2, 3}}
{{}, {1, 2}, {1, 2, 3}}

  Search stopped: reached limit of 3 families.
  Done.

Results for n=3:
Total semitopologies satisfying formula: 3
```

### Performance Characteristics

| Size | Semiframes Count | Semitopologies Count | Time (measured) | Memory Usage |
|------|------------------|---------------------|-----------------|--------------|
| n=1  | 1                | 1                   | <0.01s          | Trivial      |
| n=2  | 2                | 3                   | <0.01s          | Trivial      |
| n=3  | 10               | 14                  | <0.01s          | Trivial      |
| n=4  | 138              | 165                 | <0.01s          | Trivial          |
| n=5  | 14,005           | 14,480              | ~0.43s          | Low       |
| n=6  | ~150 million (est) | ~150 million (est) | hours?        | High (may run out of memory) |
| n=7  | ~1.5 Trillion (est) | ~1.5 Trillion (est) | years?      | Very High (hope you have a lot of ram) |

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
