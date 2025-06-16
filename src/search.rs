//! Search algorithm for semiframes and semitopologies.

use crate::canon::{Family, canonicalize, canonical_delete, family_to_str};
use crate::model_checker::{ModelChecker, Formula};
use std::collections::{HashMap, BTreeSet};
use std::fs::File;
use std::io::{Write as IoWrite, BufWriter};

#[derive(Debug)]
pub struct Config {
    pub sizes: Vec<usize>,
    pub cache_size: usize,
    pub limit: usize,
    pub output_pattern: String,
    pub search_semiframes: bool,
    pub starting_family: Option<Family>,
    pub batch_size: usize,
    pub log_interval: usize,
}

/// Checks if element p is distinguished in the given family
fn is_distinguished(family: &Family, p: usize, n: usize) -> bool {
    let p_bit = 1u32 << (p - 1);
    for q in 1..=n {
        if p == q {
            continue;
        }
        let q_bit = 1u32 << (q - 1);
        let is_separated = family.iter().any(|&s_int| {
            ((s_int & p_bit) != 0) != ((s_int & q_bit) != 0)
        });
        if !is_separated {
            return false;
        }
    }
    true
}

/// Checks if all elements are distinguished in the given family
fn has_all_distinguished(family: &Family, n: usize) -> bool {
    (1..=n).all(|p| is_distinguished(family, p, n))
}


/// Generates all canonical extensions of a family
fn extend(family: &Family, n: usize, cache: &mut HashMap<Family, Family>, max_cache_size: usize) -> Vec<Family> {
    let mut extended = BTreeSet::new();      // use a set, not a Vec

    for s_to_add in 1..(1u32 << n) {
        if family.contains(&s_to_add) {
            continue;
        }

        // upward‑closure test (as before)
        if family.iter().all(|&x| family.contains(&(x | s_to_add))) {
            let mut new_family = family.clone();
            new_family.insert(s_to_add);

            let c_new = canonicalize(&new_family, n, cache, max_cache_size);
            if canonical_delete(&c_new, n, cache, max_cache_size) == *family {
                extended.insert(c_new);      // duplicates silently ignored
            }
        }
    }
    extended.into_iter().collect()
}

fn process_and_dump_batch(
    families_to_process: &mut Vec<Family>,
    n: usize,
    outfile: &mut BufWriter<File>,
    total_found_counter: &mut usize,
    search_semiframes: bool,
    limit: usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    if families_to_process.is_empty() {
        return Ok(false);
    }
    
    print!("\r  Processing batch of {} families... Filtering...", families_to_process.len());
    std::io::stdout().flush()?;
    
    let distinguished_fams: Vec<&Family> = families_to_process
        .iter()
        .filter(|fam| {
            if search_semiframes {
                has_all_distinguished(fam, n)
            } else {
                // For semitopologies, we don't require all elements to be distinguished
                true
            }
        })
        .collect();
    
    for fam in &distinguished_fams {
        if limit > 0 && *total_found_counter >= limit {
            families_to_process.clear();
            return Ok(true); // Hit limit
        }
        // Add empty set as part of search process after distinguished point check
        let mut complete_family = (*fam).clone();
        complete_family.insert(0);
        
        writeln!(outfile, "{}", family_to_str(&complete_family, n))?;
        *total_found_counter += 1;
    }
    
    families_to_process.clear();
    
    let status_msg = format!("Batch processed. Total {} found so far: {}.", 
                           if search_semiframes { "semiframes" } else { "semitopologies" }, 
                           total_found_counter);
    print!("\r{:<80}", status_msg);
    std::io::stdout().flush()?;
    
    Ok(limit > 0 && *total_found_counter >= limit)
}

fn dfs_explore(
    family: &Family,
    n: usize,
    families_to_process: &mut Vec<Family>,
    batch_size: usize,
    outfile: &mut BufWriter<File>,
    total_found_counter: &mut usize,
    log_interval: usize,
    cache: &mut HashMap<Family, Family>,
    max_cache_size: usize,
    search_semiframes: bool,
    limit: usize,
    total_explored: &mut usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    let new_families = extend(family, n, cache, max_cache_size);
    
    for nf in new_families {
        families_to_process.push(nf.clone());
        *total_explored += 1;
        
        if *total_explored % log_interval == 0 {
            print!(
                "\r  Exploring... Total explored: {}. Batch size: {}/{}",
                *total_explored,
                families_to_process.len(),
                batch_size
            );
            std::io::stdout().flush()?;
        }
        
        if families_to_process.len() >= batch_size {
            let hit_limit = process_and_dump_batch(families_to_process, n, outfile, total_found_counter, search_semiframes, limit)?;
            if hit_limit {
                return Ok(true);
            }
        }
        
        let hit_limit = dfs_explore(
            &nf,
            n,
            families_to_process,
            batch_size,
            outfile,
            total_found_counter,
            log_interval,
            cache,
            max_cache_size,
            search_semiframes,
            limit,
            total_explored,
        )?;
        if hit_limit {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Main function to generate all families for given n with configuration
pub fn gen_fam(config: &Config, n: usize) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
    println!("--- Generating {} for n={} (Rust DFS with Batching & Caching) ---", search_type, n);
    
    let outfile_path = config.output_pattern.replace("{n}", &n.to_string());
    println!("  Batch size: {}. Log interval: {}.", config.batch_size, config.log_interval);
    println!("  Cache size: {}. Limit: {}.", 
             if config.cache_size == 0 { "disabled".to_string() } else { config.cache_size.to_string() }, 
             if config.limit == 0 { "unlimited".to_string() } else { config.limit.to_string() });
    println!("  Results will be saved to: {}", outfile_path);
    
    if n == 0 {
        return Ok((0, outfile_path));
    }

    let mut cache = HashMap::new();
    let mut families_to_process = Vec::new();
    let mut total_found_counter = 0;
    let mut total_explored = 0;
    
    let start_family = if let Some(ref custom_start) = config.starting_family {
        custom_start.clone()
    } else {
        let full_set = (1u32 << n) - 1;
        let mut family = BTreeSet::new();
        family.insert(full_set);
        family
    };
    
    println!("  Starting family: {}", family_to_str(&start_family, n));
    
    families_to_process.push(start_family.clone());

    let file = File::create(&outfile_path)?;
    let mut outfile = BufWriter::new(file);

    let hit_limit = dfs_explore(
        &start_family,
        n,
        &mut families_to_process,
        config.batch_size,
        &mut outfile,
        &mut total_found_counter,
        config.log_interval,
        &mut cache,
        config.cache_size,
        config.search_semiframes,
        config.limit,
        &mut total_explored,
    )?;

    if !hit_limit {
        println!("\n  Search complete. Processing final batch...");
        process_and_dump_batch(&mut families_to_process, n, &mut outfile, &mut total_found_counter, config.search_semiframes, config.limit)?;
    } else {
        println!("\n  Search stopped: reached limit of {} families.", config.limit);
    }

    outfile.flush()?;
    println!("  Done.");
    
    Ok((total_found_counter, outfile_path))
}

fn process_and_dump_batch_with_formula(
    families_to_process: &mut Vec<Family>,
    n: usize,
    outfile: &mut BufWriter<File>,
    total_found_counter: &mut usize,
    search_semiframes: bool,
    limit: usize,
    formula: &Formula,
) -> Result<bool, Box<dyn std::error::Error>> {
    if families_to_process.is_empty() {
        return Ok(false);
    }
    
    print!("\r  Processing batch of {} families... Filtering...", families_to_process.len());
    std::io::stdout().flush()?;
    
    let distinguished_fams: Vec<&Family> = families_to_process
        .iter()
        .filter(|fam| {
            let base_filter = if search_semiframes {
                has_all_distinguished(fam, n)
            } else {
                true
            };
            
            if !base_filter {
                return false;
            }
            
            // Add empty set as part of search process after distinguished point check
            let mut complete_family = (*fam).clone();
            complete_family.insert(0);
            
            // Check if the family satisfies the formula
            let mut checker = ModelChecker::new(n, complete_family);
            checker.check(formula).satisfied
        })
        .collect();
    
    for fam in &distinguished_fams {
        if limit > 0 && *total_found_counter >= limit {
            families_to_process.clear();
            return Ok(true); // Hit limit
        }
        writeln!(outfile, "{}", family_to_str(fam, n))?;
        *total_found_counter += 1;
    }
    
    families_to_process.clear();
    
    let status_msg = format!("Batch processed. Total {} satisfying formula found so far: {}.", 
                           if search_semiframes { "semiframes" } else { "semitopologies" }, 
                           total_found_counter);
    print!("\r{:<80}", status_msg);
    std::io::stdout().flush()?;
    
    Ok(limit > 0 && *total_found_counter >= limit)
}

fn dfs_explore_with_formula(
    family: &Family,
    n: usize,
    families_to_process: &mut Vec<Family>,
    batch_size: usize,
    outfile: &mut BufWriter<File>,
    total_found_counter: &mut usize,
    log_interval: usize,
    cache: &mut HashMap<Family, Family>,
    max_cache_size: usize,
    search_semiframes: bool,
    limit: usize,
    formula: &Formula,
    total_explored: &mut usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    let new_families = extend(family, n, cache, max_cache_size);
    
    for nf in new_families {
        families_to_process.push(nf.clone());
        *total_explored += 1;
        
        if *total_explored % log_interval == 0 {
            print!(
                "\r  Exploring... Total explored: {}. Batch size: {}/{}",
                *total_explored,
                families_to_process.len(),
                batch_size
            );
            std::io::stdout().flush()?;
        }
        
        if families_to_process.len() >= batch_size {
            let hit_limit = process_and_dump_batch_with_formula(
                families_to_process, n, outfile, total_found_counter, 
                search_semiframes, limit, formula
            )?;
            if hit_limit {
                return Ok(true);
            }
        }
        
        let hit_limit = dfs_explore_with_formula(
            &nf,
            n,
            families_to_process,
            batch_size,
            outfile,
            total_found_counter,
            log_interval,
            cache,
            max_cache_size,
            search_semiframes,
            limit,
            formula,
            total_explored,
        )?;
        if hit_limit {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Main function to generate all families satisfying a formula for given n
pub fn gen_fam_with_formula(config: &Config, n: usize, formula: &Formula) -> Result<(usize, usize, String), Box<dyn std::error::Error>> {
    let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
    println!("--- Generating {} satisfying formula for n={} (Rust DFS with Batching & Caching) ---", search_type, n);
    
    let outfile_path = config.output_pattern.replace("{n}", &n.to_string());
    println!("  Batch size: {}. Log interval: {}.", config.batch_size, config.log_interval);
    println!("  Cache size: {}. Limit: {}.", 
             if config.cache_size == 0 { "disabled".to_string() } else { config.cache_size.to_string() }, 
             if config.limit == 0 { "unlimited".to_string() } else { config.limit.to_string() });
    println!("  Results will be saved to: {}", outfile_path);
    
    if n == 0 {
        return Ok((0, 0, outfile_path));
    }

    let mut cache = HashMap::new();
    let mut families_to_process = Vec::new();
    let mut total_found_counter = 0;
    let mut total_explored = 0;
    
    let start_family = if let Some(ref custom_start) = config.starting_family {
        custom_start.clone()
    } else {
        let full_set = (1u32 << n) - 1;
        let mut family = BTreeSet::new();
        family.insert(full_set);
        family
    };
    
    println!("  Starting family: {}", family_to_str(&start_family, n));
    
    families_to_process.push(start_family.clone());

    let file = File::create(&outfile_path)?;
    let mut outfile = BufWriter::new(file);

    let hit_limit = dfs_explore_with_formula(
        &start_family,
        n,
        &mut families_to_process,
        config.batch_size,
        &mut outfile,
        &mut total_found_counter,
        config.log_interval,
        &mut cache,
        config.cache_size,
        config.search_semiframes,
        config.limit,
        formula,
        &mut total_explored,
    )?;

    if !hit_limit {
        println!("\n  Search complete. Processing final batch...");
        process_and_dump_batch_with_formula(
            &mut families_to_process, n, &mut outfile, &mut total_found_counter, 
            config.search_semiframes, config.limit, formula
        )?;
    } else {
        println!("\n  Search stopped: reached limit of {} families.", config.limit);
    }

    outfile.flush()?;
    println!("  Done.");
    
    Ok((total_found_counter, total_explored, outfile_path))
}


fn dfs_explore_with_formula_console(
    family: &Family,
    n: usize,
    total_found_counter: &mut usize,
    log_interval: usize,
    cache: &mut HashMap<Family, Family>,
    max_cache_size: usize,
    search_semiframes: bool,
    limit: usize,
    formula: &Formula,
    total_explored: &mut usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Check current family immediately
    let base_filter = if search_semiframes {
        has_all_distinguished(family, n)
    } else {
        true
    };
    
    if base_filter {
        // Add empty set as part of search process after distinguished point check
        let mut complete_family = family.clone();
        complete_family.insert(0);
        
        let mut checker = ModelChecker::new(n, complete_family.clone());
        if checker.check(formula).satisfied {
            println!("{}", family_to_str(&complete_family, n));
            *total_found_counter += 1;
            
            if limit > 0 && *total_found_counter >= limit {
                return Ok(true); // Hit limit
            }
        }
    }
    
    let new_families = extend(family, n, cache, max_cache_size);
    
    for nf in new_families {
        *total_explored += 1;
        
        if *total_explored % log_interval == 0 {
            print!(
                "\r  Exploring... Total explored: {}. Found so far: {}",
                *total_explored,
                total_found_counter
            );
            std::io::stdout().flush()?;
        }
        
        let hit_limit = dfs_explore_with_formula_console(
            &nf,
            n,
            total_found_counter,
            log_interval,
            cache,
            max_cache_size,
            search_semiframes,
            limit,
            formula,
            total_explored,
        )?;
        if hit_limit {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Main function to generate all families satisfying a formula for given n (console output)
pub fn gen_fam_with_formula_console(config: &Config, n: usize, formula: &Formula) -> Result<(usize, usize, String), Box<dyn std::error::Error>> {
    let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
    println!("--- Streaming {} satisfying formula for n={} ---", search_type, n);
    
    println!("  Log interval: {}. Cache size: {}. Limit: {}.", 
             config.log_interval,
             if config.cache_size == 0 { "disabled".to_string() } else { config.cache_size.to_string() }, 
             if config.limit == 0 { "unlimited".to_string() } else { config.limit.to_string() });
    
    if n == 0 {
        return Ok((0, 0, "console".to_string()));
    }

    let mut cache = HashMap::new();
    let mut total_found_counter = 0;
    let mut total_explored = 0;
    
    let start_family = if let Some(ref custom_start) = config.starting_family {
        custom_start.clone()
    } else {
        let full_set = (1u32 << n) - 1;
        let mut family = BTreeSet::new();
        family.insert(full_set);
        family
    };
    
    println!("  Starting family: {}", family_to_str(&start_family, n));
    println!();  // Add blank line before results

    let hit_limit = dfs_explore_with_formula_console(
        &start_family,
        n,
        &mut total_found_counter,
        config.log_interval,
        &mut cache,
        config.cache_size,
        config.search_semiframes,
        config.limit,
        formula,
        &mut total_explored,
    )?;

    if hit_limit {
        println!("\n  Search stopped: reached limit of {} families.", config.limit);
    } else {
        println!("\n  Search complete.");
    }

    println!("  Done.");
    
    Ok((total_found_counter, total_explored, "console".to_string()))
}