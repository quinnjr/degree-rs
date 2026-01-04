# degree-rs

Rust implementation of the Degree centrality plugin for PluMA.

Based on the original C++ implementation by [movingpictures83/Degree](https://github.com/movingpictures83/Degree).

## Description

Computes degree centrality for each node in a weighted adjacency matrix. Degree centrality is simply the sum of all edge weights connected to a node.

## Input Format

CSV format with nodes as rows and columns, where entry (i,j) represents the weight of the edge from node i to node j.

Example:
```csv
"","NodeA","NodeB","NodeC"
"NodeA",1,0.8,0.5
"NodeB",0.8,1,0.3
"NodeC",0.5,0.3,1
```

## Output Format

Tab-separated file with columns: Name, Centrality, Rank

Nodes are sorted by absolute centrality value in descending order.

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Usage

### Standalone CLI
```bash
cargo run --release -- example/corrP.never.csv output.txt
```

### As a library
```rust
use degree_rs::DegreePlugin;

let mut plugin = DegreePlugin::new();
plugin.input("input.csv").unwrap();
plugin.run();
plugin.output("output.txt").unwrap();
```

## License

MIT License
