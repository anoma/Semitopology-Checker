//! Search algorithm for semiframes and semitopologies.

use crate::canon::{Family, canonicalize, canonical_delete, family_to_str};
use crate::model_checker::{ModelChecker, Formula};
use std::collections::{HashMap, BTreeSet};
use std::fs::File;
use std::io::{Write as IoWrite, BufWriter};
use rayon;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use crossbeam_channel::{unbounded, Sender};

#[derive(Debug)]
pub struct Config {
    pub sizes: Vec<usize>,
    pub limit: usize,
    pub output_pattern: String,
    pub search_semiframes: bool,
    pub starting_family: Option<Family>,
    pub log_interval: usize,
    pub num_threads: usize,
}

/// Checks if element p is distinguished in the given family
pub fn is_distinguished(family: &Family, p: usize, n: usize) -> bool {
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
pub fn has_all_distinguished(family: &Family, n: usize) -> bool {
    (1..=n).all(|p| is_distinguished(family, p, n))
}

/// Main function to generate all families for given n with configuration
pub fn gen_fam(config: &Config, n: usize) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let outfile_path = config.output_pattern.replace("{n}", &n.to_string());
    let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
    
    println!("--- Generating {} for n={} (threads: {}). Writing to {} ---", 
             search_type, n, config.num_threads, outfile_path);
    
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.num_threads)
        .build_global()
        .map_err(|e| format!("Failed to initialize thread pool: {}", e))?;
    
    if n == 0 {
        return Ok((0, outfile_path));
    }

    let start_family = if let Some(ref custom_start) = config.starting_family {
        custom_start.clone()
    } else {
        let full_set = (1u32 << n) - 1;
        let mut family = BTreeSet::new();
        family.insert(full_set);
        family
    };
    
    println!("  Starting family: {}", family_to_str(&start_family, n));

    let (tx, rx) = unbounded::<Family>();
    let shared = Arc::new(SharedState {
        n,
        search_semiframes: config.search_semiframes,
        limit: config.limit,
        log_interval: config.log_interval,
        found: AtomicUsize::new(0),
        explored: AtomicUsize::new(0),
        stop: AtomicBool::new(false),
        out_tx: tx,
        formula: None,
    });

    let writer_handle = {
        let path = outfile_path.clone();
        std::thread::spawn(move || -> std::io::Result<()> {
            let mut w = BufWriter::new(File::create(path)?);
            for fam in rx {
                writeln!(w, "{}", family_to_str(&fam, n))?;
            }
            w.flush()
        })
    };

    dfs(start_family, shared.clone());

    // read the counters *before* shutting the channel
    let found = shared.found.load(Ordering::Relaxed);

    // close the channel: this drops the last Sender
    drop(shared);

    // writer thread can now finish
    writer_handle.join().unwrap()?;
    println!("\n  Done. Found {} {}.", found, search_type);
    Ok((found, outfile_path))
}

/// Shared state for parallel execution
struct SharedState<'a> {
    n: usize,
    search_semiframes: bool,
    limit: usize,
    log_interval: usize,
    found: AtomicUsize,
    explored: AtomicUsize,
    stop: AtomicBool,
    out_tx: Sender<Family>,
    formula: Option<&'a Formula>,
}

impl<'a> SharedState<'a> {
    fn try_accept(&self, fam: &Family) -> bool {
        if self.stop.load(Ordering::Relaxed) { return false; }

        let good = if self.search_semiframes {
            has_all_distinguished(fam, self.n)
        } else {
            true
        };

        if good {
            let mut complete = fam.clone();
            complete.insert(0);
            
            // Check formula if provided
            let formula_ok = if let Some(formula) = self.formula {
                let mut checker = ModelChecker::new(self.n, complete.clone());
                checker.check(formula).satisfied
            } else {
                true
            };
            
            if formula_ok {
                let new_total = self.found.fetch_add(1, Ordering::Relaxed) + 1;
                if new_total <= self.limit || self.limit == 0 {
                    self.out_tx.send(complete).ok();
                }
                if self.limit != 0 && new_total >= self.limit {
                    self.stop.store(true, Ordering::Release);
                }
            }
        }
        good
    }
}

/// Recursively explores one subtree in the Rayon pool
fn dfs(fam: Family, shared: Arc<SharedState<'_>>) {
    if shared.stop.load(Ordering::Acquire) { return; }

    let explored_count = shared.explored.fetch_add(1, Ordering::Relaxed) + 1;
    shared.try_accept(&fam);

    // Log progress periodically
    if explored_count % shared.log_interval == 0 {
        let found = shared.found.load(Ordering::Relaxed);
        print!("\r  Exploring... Total explored: {}. Found so far: {}", explored_count, found);
        std::io::stdout().flush().ok();
    }

    // Produce children inside the current thread, then recurse in parallel
    let children = extend_threadsafe(&fam, shared.n);

    rayon::scope(|s| {
        for child in children {
            let shared = shared.clone();
            s.spawn(move |_| dfs(child, shared));
        }
    });
}

/// Thread-safe version of extend (no cache, matches original logic)
fn extend_threadsafe(family: &Family, n: usize) -> Vec<Family> {
    let mut extended = BTreeSet::new();  // Use BTreeSet like the original

    for s_to_add in 1..(1u32 << n) {
        if family.contains(&s_to_add) {
            continue;
        }

        // upward-closure test (as before)
        if family.iter().all(|&x| family.contains(&(x | s_to_add))) {
            let mut new_family = family.clone();
            new_family.insert(s_to_add);

            let mut cache = HashMap::new();
            let c_new = canonicalize(&new_family, n, &mut cache, 0);
            if canonical_delete(&c_new, n, &mut cache, 0) == *family {
                extended.insert(c_new);  // duplicates silently ignored
            }
        }
    }
    extended.into_iter().collect()
}

/// Main function to generate all families satisfying a formula for given n
pub fn gen_fam_with_formula(config: &Config, n: usize, formula: &Formula) -> Result<(usize, usize, String), Box<dyn std::error::Error>> {
    let outfile_path = config.output_pattern.replace("{n}", &n.to_string());
    let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
    
    println!("--- Generating {} satisfying formula for n={} (threads: {}). Writing to {} ---", 
             search_type, n, config.num_threads, outfile_path);
    
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.num_threads)
        .build_global()
        .map_err(|e| format!("Failed to initialize thread pool: {}", e))?;
    
    if n == 0 {
        return Ok((0, 0, outfile_path));
    }

    let start_family = if let Some(ref custom_start) = config.starting_family {
        custom_start.clone()
    } else {
        let full_set = (1u32 << n) - 1;
        let mut family = BTreeSet::new();
        family.insert(full_set);
        family
    };
    
    println!("  Starting family: {}", family_to_str(&start_family, n));

    let (tx, rx) = unbounded::<Family>();
    let shared = Arc::new(SharedState {
        n,
        search_semiframes: config.search_semiframes,
        limit: config.limit,
        log_interval: config.log_interval,
        found: AtomicUsize::new(0),
        explored: AtomicUsize::new(0),
        stop: AtomicBool::new(false),
        out_tx: tx,
        formula: Some(formula),
    });

    let writer_handle = {
        let path = outfile_path.clone();
        std::thread::spawn(move || -> std::io::Result<()> {
            let mut w = BufWriter::new(File::create(path)?);
            for fam in rx {
                writeln!(w, "{}", family_to_str(&fam, n))?;
            }
            w.flush()
        })
    };

    dfs(start_family, shared.clone());

    // read the counters *before* shutting the channel
    let found = shared.found.load(Ordering::Relaxed);
    let explored = shared.explored.load(Ordering::Relaxed);

    // close the channel: this drops the last Sender
    drop(shared);

    // writer thread can now finish
    writer_handle.join().unwrap()?;
    println!("\n  Done. Found {} {} satisfying formula.", found, search_type);
    Ok((found, explored, outfile_path))
}



/// Main function to generate all families satisfying a formula for given n (console output)
pub fn gen_fam_with_formula_console(config: &Config, n: usize, formula: &Formula, quiet: bool) -> Result<(usize, usize, String), Box<dyn std::error::Error>> {
    let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
    
    println!("--- Streaming {} satisfying formula for n={} (threads: {}) ---", search_type, n, config.num_threads);
    
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.num_threads)
        .build_global()
        .map_err(|e| format!("Failed to initialize thread pool: {}", e))?;
    
    if n == 0 {
        return Ok((0, 0, "console".to_string()));
    }

    let start_family = if let Some(ref custom_start) = config.starting_family {
        custom_start.clone()
    } else {
        let full_set = (1u32 << n) - 1;
        let mut family = BTreeSet::new();
        family.insert(full_set);
        family
    };
    
    println!("  Starting family: {}", family_to_str(&start_family, n));
    println!();

    let (tx, rx) = unbounded::<Family>();
    let shared = Arc::new(SharedState {
        n,
        search_semiframes: config.search_semiframes,
        limit: config.limit,
        log_interval: config.log_interval,
        found: AtomicUsize::new(0),
        explored: AtomicUsize::new(0),
        stop: AtomicBool::new(false),
        out_tx: tx,
        formula: Some(formula),
    });

    let writer_handle = std::thread::spawn(move || {
        for fam in rx {
            if !quiet {
                println!("{}", family_to_str(&fam, n));
            }
        }
    });

    dfs(start_family, shared.clone());

    // read the counters *before* shutting the channel
    let found = shared.found.load(Ordering::Relaxed);
    let explored = shared.explored.load(Ordering::Relaxed);

    // close the channel: this drops the last Sender
    drop(shared);

    // writer thread can now finish
    writer_handle.join().unwrap();
    
    if config.limit != 0 && found >= config.limit {
        println!("\n  Search stopped: reached limit of {} families.", config.limit);
    } else {
        println!("\n  Search complete.");
    }
    println!("  Done.");
    
    Ok((found, explored, "console".to_string()))
}