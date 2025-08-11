use std::process::exit;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use glob::glob;
use stapler::merge::FileSystemOptions;
use stapler::stapler;

fn expand_glob_patterns(patterns: Vec<String>) -> Result<Vec<String>> {
    let mut expanded_files = Vec::new();
    
    for pattern in patterns {
        // Check if the pattern contains glob characters
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            // It's a glob pattern, expand it
            let glob_matches = glob(&pattern)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
            
            let mut pattern_matches = Vec::new();
            for entry in glob_matches {
                match entry {
                    Ok(path) => {
                        // Only include files that exist and have .pdf extension
                        if path.is_file() && path.extension()
                            .map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "pdf") {
                            pattern_matches.push(path.to_string_lossy().to_string());
                        }
                    }
                    Err(e) => {
                        eprintln!("[STAPLER] Warning: Error processing glob pattern '{}': {}", pattern, e);
                    }
                }
            }
            
            if pattern_matches.is_empty() {
                eprintln!("[STAPLER] Warning: No PDF files found matching pattern: {}", pattern);
            } else {
                // Sort the matches for consistent ordering
                pattern_matches.sort();
                expanded_files.extend(pattern_matches);
            }
        } else {
            // It's a literal file path, add it directly
            expanded_files.push(pattern);
        }
    }
    
    Ok(expanded_files)
}

fn parse_cli_arguments() -> Result<(Vec<String>, String, bool)> {
    let matches = Command::new("stapler")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILES")
                .help("Input PDF files or glob patterns (e.g., *.pdf, /path/to/*.pdf)")
                .num_args(1..)
                .value_delimiter(' ')
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output PDF file")
                .required(true),
        )
        .arg(
            Arg::new("compress")
                .action(ArgAction::SetTrue)
                .short('c')
                .long("compress")
                .help("Compress the output PDF file")
                .required(false),
        )
        .get_matches();

    let input_patterns: Vec<String> = matches
        .get_many::<String>("input")
        .context("No input files provided")?
        .cloned()
        .collect();

    // Expand glob patterns
    let input_files = expand_glob_patterns(input_patterns)?;
    
    if input_files.is_empty() {
        anyhow::bail!("No PDF files found after expanding patterns");
    }
    
    if input_files.len() < 2 {
        anyhow::bail!("At least 2 PDF files are required for merging");
    }

    let output_file: String = matches
        .get_one::<String>("output")
        .context("No output file provided")?
        .clone();

    let compress = matches
        .get_one::<bool>("compress")
        .copied()
        .unwrap_or(false);

    Ok((input_files, output_file, compress))
}

fn main() -> Result<()> {
    let (input_files, output_file, compress) = parse_cli_arguments()?;
    let file_options = FileSystemOptions::from((&input_files, &output_file, compress));

    println!(
        "[STAPLER] Found {} PDF files to merge into {}",
        input_files.len(), output_file
    );
    
    if input_files.len() <= 10 {
        println!("[STAPLER] Input files: {:?}", input_files);
    } else {
        println!("[STAPLER] Input files: {} files (showing first 5): {:?}...", 
                 input_files.len(), &input_files[..5]);
    }

    if let Err(e) = stapler(file_options) {
        eprintln!("[STAPLER] Error: {}", e);
        exit(1);
    }

    println!(
        "[STAPLER] PDFs merged successfully. Output file: {}",
        output_file
    );

    Ok(())
}
