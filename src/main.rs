//! Degree Centrality CLI
//!
//! Standalone command-line interface for degree centrality computation.

use degree_rs::DegreePlugin;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.csv> <output.txt>", args[0]);
        eprintln!("\nComputes degree centrality for a weighted adjacency matrix.");
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    let mut plugin = DegreePlugin::new();

    if let Err(e) = plugin.input(input_file) {
        eprintln!("Error reading input: {}", e);
        process::exit(1);
    }

    println!("[DegreePlugin] Processing {} nodes", plugin.size());

    plugin.run();

    println!("[DegreePlugin] Computed degree centrality");

    if let Err(e) = plugin.output(output_file) {
        eprintln!("Error writing output: {}", e);
        process::exit(1);
    }

    println!("[DegreePlugin] Results written to {}", output_file);
}
