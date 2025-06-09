import functools
import time
import pynauty
import sys
from typing import Set, FrozenSet, List, Tuple

# Type alias for a family of subsets, represented as a frozenset of integers.
Family = FrozenSet[int]

# Define a constant for the single, most effective cache
CACHE_SIZE = 2**20

# --- HELPER FUNCTIONS ---

def set_to_int(s: set[int]) -> int:
    """Converts a set of 1-based integers to its bitmask representation."""
    val = 0
    for x in s:
        val |= (1 << (x - 1))
    return val

def int_to_set(i: int, n: int) -> Set[int]:
    """Converts a bitmask back to a set of 1-based integers."""
    s = set()
    for j in range(n):
        if (i >> j) & 1:
            s.add(j + 1)
    return s

def family_to_str(l: Family, n: int) -> str:
    """Creates a human-readable string for a family of sets."""
    if not l:
        return "{}"
    sorted_ints = sorted(list(l))
    set_list = [tuple(sorted(list(int_to_set(i, n)))) for i in sorted_ints]
    set_list.sort(key=lambda x: (len(x), x))
    return "{" + ", ".join(str(set(s)) for s in set_list) + "}"

# --- COMPUTATION FUNCTIONS (CACHE REMOVED WHERE INEFFICIENT) ---

# No cache: This function is called only once per family during batch processing.
def is_distinguished(l: Family, p: int, n: int) -> bool:
    p_bit = 1 << (p - 1)
    for q in range(1, n + 1):
        if p == q:
            continue
        q_bit = 1 << (q - 1)
        is_separated = any((((s_int & p_bit) != 0) != ((s_int & q_bit) != 0)) for s_int in l)
        if not is_separated:
            return False
    return True

def has_all_distinguished(l: Family, n: int) -> bool:
    return all(is_distinguished(l, p, n) for p in range(1, n + 1))

def family_to_bipartite_graph(l: Family, n: int) -> pynauty.Graph:
    sets_list = list(l)
    num_sets = len(sets_list)
    total_vertices = n + num_sets
    adjacency = {i: [] for i in range(total_vertices)}
    for set_idx, s_int in enumerate(sets_list):
        set_vertex = n + set_idx
        for element_idx in range(n):
            if (s_int >> element_idx) & 1:
                element_vertex = element_idx
                adjacency[element_vertex].append(set_vertex)
                adjacency[set_vertex].append(element_vertex)
    graph = pynauty.Graph(total_vertices, directed=False, adjacency_dict=adjacency)
    element_vertices = set(range(n))
    set_vertices = set(range(n, n + num_sets))
    vertex_coloring = [element_vertices, set_vertices]
    if element_vertices and set_vertices:
        graph.set_vertex_coloring(vertex_coloring)
    return graph

# Cache kept: This is the core bottleneck and is called repeatedly with the same inputs.
@functools.lru_cache(maxsize=CACHE_SIZE)
def canonicalize(l: Family, n: int) -> Family:
    if not l:
        return frozenset()
    graph = family_to_bipartite_graph(l, n)
    canonical_labeling = pynauty.canon_label(graph)
    element_permutation = canonical_labeling[:n]
    canonical_family = set()
    for s_int in l:
        new_s_int = 0
        for i in range(n):
            if (s_int >> i) & 1:
                canonical_pos = element_permutation.index(i)
                new_s_int |= (1 << canonical_pos)
        canonical_family.add(new_s_int)
    return frozenset(canonical_family)

# No cache: This function is lightweight and its expense comes from canonicalize, which is already cached.
def canonical_delete(l: Family, n: int) -> Family:
    if not l:
        return frozenset()
    temp_list = sorted(list(l))
    if len(temp_list) <= 1:
        return frozenset()
    return canonicalize(frozenset(temp_list[1:]), n)

def extend(l: Family, n: int) -> Set[Family]:
    extended_families = set()
    for s_to_add in range(1, 1 << n):
        if s_to_add in l:
            continue
        is_ideal_extension = all((x | s_to_add) in l for x in l)
        if is_ideal_extension:
            new_family = l.union({s_to_add})
            c_new_family = canonicalize(new_family, n)
            if canonical_delete(c_new_family, n) == l:
                extended_families.add(c_new_family)
    return extended_families

# --- BATCH PROCESSING AND EXPLORATION LOGI ---

def process_and_dump_batch(
    families_to_process: List[Family],
    n: int,
    outfile: "TextIOWrapper",
    total_found_counter: List[int]
):
    if not families_to_process:
        return
    print(f"\r  Processing batch of {len(families_to_process)} families... Filtering...", end="", flush=True)
    distinguished_fams = [fam for fam in families_to_process if has_all_distinguished(fam, n)]
    if distinguished_fams:
        for fam in distinguished_fams:
            outfile.write(family_to_str(fam, n) + '\n')
    total_found_counter[0] += len(distinguished_fams)
    families_to_process.clear()
    status_msg = f"Batch processed. Total distinguished found so far: {total_found_counter[0]}."
    print(f"\r{status_msg:<80}", flush=True)

def dfs_explore(
    family: Family,
    n: int,
    visited_families: Set[Family],
    families_to_process: List[Family],
    batch_size: int,
    outfile: "TextIOWrapper",
    total_found_counter: List[int],
    log_interval: int
):
    new_families = extend(family, n)
    for nf in new_families:
        if nf not in visited_families:
            visited_families.add(nf)
            families_to_process.append(nf)
            if len(visited_families) % log_interval == 0:
                print(f"\r  Exploring... Total visited: {len(visited_families)}. Batch size: {len(families_to_process)}/{batch_size}", end="", flush=True)
            if len(families_to_process) >= batch_size:
                process_and_dump_batch(families_to_process, n, outfile, total_found_counter)
            dfs_explore(nf, n, visited_families, families_to_process, batch_size, outfile, total_found_counter, log_interval)

def gen_fam(n: int, batch_size: int = 100000, log_interval: int = 1000) -> Tuple[int, str]:
    """
    Generates all distinguished families for 'n', processing them in batches to
    save memory and writing the results to a file.
    """
    print(f"--- Generating for n={n} (DFS with Batching & Refined Caching) ---")
    outfile_path = f"distinguished_families_n{n}.txt"
    print(f"  Batch size: {batch_size}. Log interval: {log_interval}. Cache size for canonicalize: {CACHE_SIZE}")
    print(f"  Results will be saved to: {outfile_path}")
    
    if n == 0:
        return 0, outfile_path

    # Clear the single, essential cache for each run of gen_fam
    canonicalize.cache_clear()

    visited_families = set()
    families_to_process = []
    total_found_counter = [0]
    full_set = (1 << n) - 1
    start_family = frozenset({full_set})
    visited_families.add(start_family)
    families_to_process.append(start_family)

    with open(outfile_path, 'w') as outfile:
        dfs_explore(
            start_family, n, visited_families, families_to_process,
            batch_size, outfile, total_found_counter, log_interval
        )
        print("\n  Search complete. Processing final batch...")
        process_and_dump_batch(families_to_process, n, outfile, total_found_counter)

    print("  Done.")
    return total_found_counter[0], outfile_path

if __name__ == '__main__':
    total_start_time = time.time()
    for n_val in range(1, 6):
        start_time = time.time()
        count, filename = gen_fam(n_val)
        end_time = time.time()
        print(f"\nResults for n={n_val}:")
        print(f"Total distinguished families found: {count}")
        print(f"Results saved in: {filename}")
        print(f"Time taken: {end_time - start_time:.3f} seconds")
        print("-" * 20 + "\n")
    total_end_time = time.time()
    print(f"Total execution time: {total_end_time - total_start_time:.3f} seconds")
