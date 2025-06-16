//! Main entry point for the semiframes and semitopologies checker.

mod search;
mod canon;
mod model_checker;
mod parser;
mod tokens;
mod ast;
mod macro_expander;

use clap::{Parser, Subcommand};
use search::{Config, gen_fam};
use canon::{Family, parse_family_str, canonicalize_once, family_to_str, infer_size_from_family};
use model_checker::{ModelChecker, Witness};
use parser::parse_formula;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "semiframes")]
#[command(about = "A tool for finding semiframes and semitopologies")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for semiframes or semitopologies
    Search {
        /// Size to search for (single number or range like "3-5")
        #[arg(short = 's', long, default_value = "1-6")]
        size: String,

        /// Maximum cache size (0 to disable caching)
        #[arg(short = 'c', long, default_value = "10000")]
        cache_size: usize,

        /// Hard limit on number of families to generate (0 for unlimited)
        #[arg(short = 'l', long, default_value = "0")]
        limit: usize,

        /// Output file name pattern (use {n} for size placeholder)
        #[arg(short = 'o', long, default_value = "distinguished_families_n{n}.txt")]
        output: String,

        /// Search for semiframes instead of semitopologies
        #[arg(long)]
        semiframes: bool,

        /// Starting family as semitopology (e.g., "{{1}, {1,2}, {1,2,3}}")
        #[arg(long)]
        starting_family: Option<String>,

        /// Batch size for processing
        #[arg(short = 'b', long, default_value = "100000")]
        batch_size: usize,

        /// Log interval for progress reporting
        #[arg(long, default_value = "10000")]
        log_interval: usize,

        /// Number of threads to use (1 for sequential, >1 for parallel)
        #[arg(short = 't', long = "threads", default_value = "1")]
        threads: usize,
    },
    /// Canonicalize a given semitopology
    Canon {
        /// The semitopology to canonicalize (e.g., "{{1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}")
        #[arg(short = 'f', long)]
        family: String,

        /// Size n for the semitopology (auto-inferred if not provided)
        #[arg(short = 'n', long)]
        size: Option<usize>,
    },
    /// Check if a semitopology satisfies a given formula
    Check {
        /// The formula to check (e.g., "AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)")
        #[arg(short = 'f', long)]
        formula: String,
        
        /// The semitopology to check against (e.g., "{{1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}")
        #[arg(short = 's', long)]
        semitopology: String,

        /// Size n for the semitopology (auto-inferred if not provided)
        #[arg(short = 'n', long)]
        size: Option<usize>,
    },
    /// Find semitopologies that satisfy a given formula
    Find {
        /// The formula to satisfy (e.g., "AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)")
        #[arg(short = 'f', long)]
        formula: String,
        
        /// Size to search for (single number or range like "3-5")
        #[arg(short = 's', long, default_value = "1-6")]
        size: String,

        /// Maximum cache size (0 to disable caching)
        #[arg(short = 'c', long, default_value = "10000")]
        cache_size: usize,

        /// Hard limit on number of families to generate
        #[arg(short = 'l', long, default_value = "1")]
        limit: usize,

        /// Output file name pattern (use {n} for size placeholder, optional)
        #[arg(short = 'o', long)]
        output: Option<String>,

        /// Search for semiframes instead of semitopologies
        #[arg(long)]
        semiframes: bool,

        /// Starting family as semitopology (e.g., "{{1}, {1,2}, {1,2,3}}")
        #[arg(long)]
        starting_family: Option<String>,

        /// Batch size for processing
        #[arg(short = 'b', long, default_value = "100000")]
        batch_size: usize,

        /// Log interval for progress reporting
        #[arg(long, default_value = "10000")]
        log_interval: usize,

        /// Number of threads to use (1 for sequential, >1 for parallel)
        #[arg(short = 't', long = "threads", default_value = "1")]
        threads: usize,

        /// Suppress printing of found semitopologies (only show count)
        #[arg(short = 'q', long)]
        quiet: bool,
    },
}

fn parse_size_range(size_str: &str) -> Result<Vec<usize>, String> {
    if size_str.contains('-') {
        let parts: Vec<&str> = size_str.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range format: {}", size_str));
        }
        let start: usize = parts[0].parse().map_err(|_| format!("Invalid start: {}", parts[0]))?;
        let end: usize = parts[1].parse().map_err(|_| format!("Invalid end: {}", parts[1]))?;
        if start > end {
            return Err(format!("Start {} is greater than end {}", start, end));
        }
        Ok((start..=end).collect())
    } else {
        let single: usize = size_str.parse().map_err(|_| format!("Invalid number: {}", size_str))?;
        Ok(vec![single])
    }
}

fn parse_starting_family(family_str: &str, n: usize) -> Result<Family, String> {
    // Use the same parsing logic as the canon command
    let family = parse_family_str(family_str, n)
        .map_err(|e| format!("Invalid starting family format: {}", e))?;
    
    // Canonicalize the starting family
    let canonical_family = canonicalize_once(&family, n);
    
    Ok(canonical_family)
}

fn parse_search_args(
    size: String,
    limit: usize,
    output: String,
    semiframes: bool,
    starting_family: Option<String>,
    log_interval: usize,
    threads: usize,
) -> Result<Config, String> {
    let sizes = parse_size_range(&size)?;
    
    let starting_family = if let Some(ref family_str) = starting_family {
        if sizes.len() == 1 {
            Some(parse_starting_family(family_str, sizes[0])?)
        } else {
            return Err("Starting family can only be specified for single size, not range".to_string());
        }
    } else {
        None
    };
    
    Ok(Config {
        sizes,
        limit,
        output_pattern: output,
        search_semiframes: semiframes,
        starting_family,
        log_interval,
        num_threads: threads,
    })
}

fn handle_search_command(
    size: String,
    limit: usize,
    output: String,
    semiframes: bool,
    starting_family: Option<String>,
    log_interval: usize,
    threads: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_search_args(
        size, limit, output, semiframes,
        starting_family, log_interval, threads
    ).map_err(|e| format!("Error parsing arguments: {}", e))?;
    
    let total_start_time = Instant::now();
    
    for n_val in &config.sizes {
        let start_time = Instant::now();
        let (count, filename) = gen_fam(&config, *n_val)?;
        let end_time = Instant::now();
        
        println!("\nResults for n={}:", n_val);
        let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
        println!("Total {} found: {}", search_type, count);
        println!("Results saved in: {}", filename);
        println!("Time taken: {:.3} seconds", (end_time - start_time).as_secs_f64());
        println!("{}", "-".repeat(50));
    }
    
    let total_end_time = Instant::now();
    println!("Total execution time: {:.3} seconds", (total_end_time - total_start_time).as_secs_f64());
    
    Ok(())
}

fn handle_canon_command(family_str: String, size: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    // First, try to infer size from the family if not provided
    let temp_family = parse_family_str(&family_str, 32) // Use max possible size for parsing
        .map_err(|e| format!("Error parsing family: {}", e))?;
    
    let n = size.unwrap_or_else(|| infer_size_from_family(&temp_family));
    
    if n == 0 {
        return Err("Could not determine size n. Please specify with --size or ensure family contains at least one non-empty set.".into());
    }
    
    // Parse the family properly with the correct size
    let family = parse_family_str(&family_str, n)
        .map_err(|e| format!("Error parsing family: {}", e))?;
    
    println!("Input family (n={}): {}", n, family_to_str(&family, n));
    
    let canonical_family = canonicalize_once(&family, n);
    
    println!("Canonical form: {}", family_to_str(&canonical_family, n));
    
    Ok(())
}

fn handle_check_command(formula_str: String, semitopology_str: String, size: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the formula
    let formula = parse_formula(&formula_str)
        .map_err(|e| format!("Error parsing formula: {}", e))?;
    
    // First, try to infer size from the family if not provided
    let temp_family = parse_family_str(&semitopology_str, 32) // Use max possible size for parsing
        .map_err(|e| format!("Error parsing semitopology: {}", e))?;
    
    let n = size.unwrap_or_else(|| infer_size_from_family(&temp_family));
    
    if n == 0 {
        return Err("Could not determine size n. Please specify with --size or ensure family contains at least one non-empty set.".into());
    }
    
    // Parse the family properly with the correct size
    let family = parse_family_str(&semitopology_str, n)
        .map_err(|e| format!("Error parsing semitopology: {}", e))?;
    
    println!("Formula: {}", formula_str);
    println!("Semitopology (n={}): {}", n, family_to_str(&family, n));
    
    // Create model checker and check the formula
    let mut checker = ModelChecker::new(n, family);
    let result = checker.check(&formula);
    
    if result.satisfied {
        println!("Result: ✓ SATISFIED");
        
        if !result.witnesses.is_empty() {
            println!("Witnesses:");
            for (var, witness) in result.witnesses {
                match witness {
                    Witness::Point(p) => println!("  {} = point {}", var, p),
                    Witness::Open(mask) => {
                        let mut open_points = Vec::new();
                        for i in 0..n {
                            if (mask >> i) & 1 == 1 {
                                open_points.push(i + 1);
                            }
                        }
                        println!("  {} = {{{}}}", var, open_points.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "));
                    }
                }
            }
        }
    } else {
        println!("Result: ✗ NOT SATISFIED");
    }
    
    Ok(())
}

fn handle_find_command(
    formula_str: String,
    size: String,
    limit: usize,
    output: Option<String>,
    semiframes: bool,
    starting_family: Option<String>,
    log_interval: usize,
    threads: usize,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the formula first
    let formula = parse_formula(&formula_str)
        .map_err(|e| format!("Error parsing formula: {}", e))?;
    
    println!("Searching for semitopologies satisfying formula: {}", formula_str);
    
    // Determine if we should output to file or console
    let output_to_file = output.is_some();
    let output_pattern = output.unwrap_or_else(|| "console".to_string());
    
    // Create a modified config that includes the formula
    let config = parse_search_args(
        size, limit, output_pattern, semiframes,
        starting_family, log_interval, threads
    ).map_err(|e| format!("Error parsing arguments: {}", e))?;
    
    let total_start_time = Instant::now();
    
    for n_val in &config.sizes {
        let start_time = Instant::now();
        let (results, explored, filename) = if output_to_file {
            search::gen_fam_with_formula(&config, *n_val, &formula)?
        } else {
            search::gen_fam_with_formula_console(&config, *n_val, &formula, quiet)?
        };
        let end_time = Instant::now();
        
        println!("\nResults for n={}:", n_val);
        let search_type = if config.search_semiframes { "semiframes" } else { "semitopologies" };
        
        println!("Total {} explored: {}", search_type, explored);
        if output_to_file {
            println!("Total {} satisfying formula: {}", search_type, results);
            println!("Results saved in: {}", filename);
        } else {
            println!("Total {} satisfying formula: {}", search_type, results);
        }
        
        println!("Time taken: {:.3} seconds", (end_time - start_time).as_secs_f64());
        println!("{}", "-".repeat(50));
    }
    
    let total_end_time = Instant::now();
    println!("Total execution time: {:.3} seconds", (total_end_time - total_start_time).as_secs_f64());
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    match args.command {
        Commands::Search { 
            size, limit, output, semiframes, 
            starting_family, log_interval, threads, ..
        } => {
            handle_search_command(
                size, limit, output, semiframes,
                starting_family, log_interval, threads
            )
        }
        Commands::Canon { family, size } => {
            handle_canon_command(family, size)
        }
        Commands::Check { formula, semitopology, size } => {
            handle_check_command(formula, semitopology, size)
        }
        Commands::Find { 
            formula, size, limit, output, semiframes, 
            starting_family, log_interval, threads, quiet, ..
        } => {
            handle_find_command(
                formula, size, limit, output, semiframes,
                starting_family, log_interval, threads, quiet
            )
        }
    }
}