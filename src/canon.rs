//! Canonicalization module using nauty for canonical graph labeling.

use nauty_Traces_sys::{
    densenauty, 
    optionblk, statsblk, setword, graph, SETWORDSNEEDED,
};
use std::collections::{HashMap, HashSet, BTreeSet};
use std::os::raw::c_int;

/// A family of subsets represented as a set of bitmasks
pub type Family = BTreeSet<u32>;

/// Converts a bitmask back to a set of 1-based integers
fn int_to_set(i: u32, n: usize) -> HashSet<usize> {
    let mut s = HashSet::new();
    for j in 0..n {
        if (i >> j) & 1 == 1 {
            s.insert(j + 1);
        }
    }
    s
}

/// Creates a human-readable string representation of a family of sets
pub fn family_to_str(family: &Family, n: usize) -> String {
    if family.is_empty() {
        return "{}".to_string();
    }
    
    let mut sorted_ints: Vec<u32> = family.iter().cloned().collect();
    sorted_ints.sort();
    
    let mut set_list: Vec<Vec<usize>> = sorted_ints
        .iter()
        .map(|&i| {
            let mut set: Vec<usize> = int_to_set(i, n).into_iter().collect();
            set.sort();
            set
        })
        .collect();
    
    set_list.sort_by_key(|s| (s.len(), s.clone()));
    
    let set_strings: Vec<String> = set_list
        .iter()
        .map(|s| {
            if s.is_empty() {
                "{}".to_string()
            } else {
                format!("{{{}}}", s.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
            }
        })
        .collect();
    
    format!("{{{}}}", set_strings.join(", "))
}

/// Parses a family string like "{{1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}" into a Family
pub fn parse_family_str(family_str: &str, n: usize) -> Result<Family, String> {
    let mut family = BTreeSet::new();
    
    // Remove outer braces and whitespace
    let trimmed = family_str.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("Family must be enclosed in outer braces like {{1,2},{3}}".to_string());
    }
    
    let inner = &trimmed[1..trimmed.len()-1].trim();
    if inner.is_empty() {
        return Ok(family); // Empty family
    }
    
    // Parse individual sets
    let mut chars = inner.chars().peekable();
    let mut current_set = String::new();
    let mut brace_count = 0;
    
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                brace_count += 1;
                current_set.push(ch);
            }
            '}' => {
                brace_count -= 1;
                current_set.push(ch);
                if brace_count == 0 {
                    // End of a set, parse it
                    let set_mask = parse_single_set(&current_set, n)?;
                    family.insert(set_mask);
                    current_set.clear();
                    
                    // Skip comma and whitespace
                    while chars.peek() == Some(&',') || chars.peek() == Some(&' ') {
                        chars.next();
                    }
                }
            }
            _ => {
                if brace_count > 0 {
                    current_set.push(ch);
                }
            }
        }
    }
    
    Ok(family)
}

/// Parses a single set string like "{1, 2, 3}" into a bitmask
fn parse_single_set(set_str: &str, n: usize) -> Result<u32, String> {
    let trimmed = set_str.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(format!("Set must be enclosed in braces: {}", set_str));
    }
    
    let inner = &trimmed[1..trimmed.len()-1].trim();
    if inner.is_empty() {
        return Ok(0); // Empty set
    }
    
    let mut mask = 0u32;
    for element_str in inner.split(',') {
        let element: usize = element_str.trim().parse()
            .map_err(|_| format!("Invalid element: {}", element_str))?;
        if element == 0 || element > n {
            return Err(format!("Element {} is out of range for n={}", element, n));
        }
        let bit_pos = element - 1;
        mask |= 1u32 << bit_pos;
    }
    
    Ok(mask)
}

/// Builds a bipartite graph for nauty with element vertices and set vertices
fn build_dense_bipartite(sets: &[u32], n: usize) -> (Vec<setword>, usize) {
    let v = n + sets.len();
    let m = SETWORDSNEEDED(v);

    let mut g = vec![0 as setword; v * m];

    let set_edge = |g: &mut [setword], m: usize, i: usize, j: usize| {
        let row_start = i * m;
        let slice = &mut g[row_start..row_start + m];
        nauty_Traces_sys::ADDELEMENT(slice, j);
    };

    for (k, &mask) in sets.iter().enumerate() {
        let set_vertex = n + k;
        for element_idx in 0..n {
            if (mask >> element_idx) & 1 == 1 {
                let element_vertex = element_idx;
                set_edge(&mut g, m, element_vertex, set_vertex);
                set_edge(&mut g, m, set_vertex, element_vertex);
            }
        }
    }
    (g, m)
}

/// Computes canonical permutation using nauty
fn canon_permutation(sets: &[u32], n: usize) -> Vec<usize> {
    let (mut g, m) = build_dense_bipartite(sets, n);
    let v = n + sets.len();

    // Set up vertex coloring: element vertices (0..n-1) vs set vertices (n..v-1)
    let mut lab: Vec<c_int> = Vec::new();
    let mut ptn: Vec<c_int> = Vec::new();
    
    // Add element vertices first
    for i in 0..n {
        lab.push(i as c_int);
        ptn.push(1);  // 1 means "not end of partition"
    }
    if n > 0 {
        ptn[n - 1] = 0;  // 0 means "end of partition"
    }
    
    // Add set vertices
    for i in n..v {
        lab.push(i as c_int);
        ptn.push(1);  // 1 means "not end of partition"
    }
    if v > n {
        ptn[v - 1] = 0;  // 0 means "end of partition"
    }
    
    let mut orbits = vec![0 as c_int; v];
    let mut options: optionblk = optionblk::default();
    let mut stats: statsblk = unsafe { std::mem::zeroed() };
    
    options.getcanon = 1;
    options.defaultptn = 0;  // CRITICAL: Use our custom partition!

    let mut canon = vec![0 as setword; v * m];
    
    unsafe {
        densenauty(
            g.as_mut_ptr() as *mut graph,
            lab.as_mut_ptr(),
            ptn.as_mut_ptr(),
            orbits.as_mut_ptr(),
            &mut options,
            &mut stats,
            m as c_int,
            v as c_int,
            canon.as_mut_ptr() as *mut graph,
        );
    }

    lab.iter().map(|&x| x as usize).collect()
}

/// Canonicalizes a family using nauty with caching
pub fn canonicalize(family: &Family, n: usize, cache: &mut HashMap<Family, Family>, max_cache_size: usize) -> Family {
    if family.is_empty() {
        return BTreeSet::new();
    }
    
    if max_cache_size > 0 {
        if let Some(cached) = cache.get(family) {
            return cached.clone();
        }
    }

    let sets: Vec<u32> = family.iter().cloned().collect();
    let canonical_labeling = canon_permutation(&sets, n);
    
    // Python code does: element_permutation = canonical_labeling[:n]
    // This should be a permutation of [0, 1, ..., n-1]
    let element_permutation = &canonical_labeling[..n];
    
    // Verify that element_permutation is a valid permutation of [0, 1, ..., n-1]
    let mut sorted_elements: Vec<usize> = element_permutation.to_vec();
    sorted_elements.sort();
    let expected: Vec<usize> = (0..n).collect();
    if sorted_elements != expected {
        panic!("Invalid element permutation: {:?}", element_permutation);
    }
    
    let mut canonical_family = BTreeSet::new();
    for &s_int in family {
        let mut new_s_int = 0u32;
        for i in 0..n {
            if (s_int >> i) & 1 == 1 {
                let canonical_pos = element_permutation.iter().position(|&x| x == i).unwrap();
                new_s_int |= 1 << canonical_pos;
            }
        }
        canonical_family.insert(new_s_int);
    }
    
    if max_cache_size > 0 {
        if cache.len() >= max_cache_size {
            cache.clear();
        }
        cache.insert(family.clone(), canonical_family.clone());
    }
    canonical_family
}

/// Canonicalizes a family without caching (for one-off canonicalization)
pub fn canonicalize_once(family: &Family, n: usize) -> Family {
    let mut dummy_cache = HashMap::new();
    canonicalize(family, n, &mut dummy_cache, 0)
}

/// Removes the lexicographically largest set and canonicalizes
pub fn canonical_delete(family: &Family, n: usize, cache: &mut HashMap<Family, Family>, max_cache_size: usize) -> Family {
    if family.is_empty() {
        return BTreeSet::new();
    }
    
    let mut temp_list: Vec<u32> = family.iter().cloned().collect();
    temp_list.sort();
    
    if temp_list.len() <= 1 {
        return BTreeSet::new();
    }
    
    let reduced_family: BTreeSet<u32> = temp_list[1..].iter().cloned().collect();
    canonicalize(&reduced_family, n, cache, max_cache_size)
}

/// Infers the size n from a family by finding the maximum element
pub fn infer_size_from_family(family: &Family) -> usize {
    let mut max_element = 0;
    for &mask in family {
        for i in 0..32 { // u32 has 32 bits
            if (mask >> i) & 1 == 1 {
                max_element = max_element.max(i + 1);
            }
        }
    }
    max_element
}