# degree-rs example

A real-world fixture for exercising the Degree centrality plugin: a 126 × 126
signed correlation network of oral-microbiome OTUs from the corncob /
SparCC-style pipeline used by the [`movingpictures83/Degree`](https://github.com/movingpictures83/Degree)
upstream tests.

## Files

| File | Shape | Contents |
|---|---|---|
| `corrP.never.csv` | 126 × 126 | Weighted adjacency matrix. First row + first column are OTU labels; cell `(i, j)` is the signed correlation weight (or 0 for absent edges). |
| `output.txt` | 126 × 3 | Reference output (`Name`, `Centrality`, `Rank`) from the original C++ Degree plugin — useful for diff-style regression checks. |

## Running the example

### Via the CLI binary

```bash
cargo run --release -- example/corrP.never.csv /tmp/degree.out
diff /tmp/degree.out example/output.txt
```

Should produce an empty diff modulo floating-point noise (centrality values
are deterministic single-precision sums; ordering may differ on ties).

### Via PluMA (as a real Rust plugin)

Once [`FIUBioRG/PluMA#13`](https://github.com/FIUBioRG/PluMA/pull/13) lands so
`pluma` is built with `-DHAVE_RUST`, the plugin loads under:

```text
PluMA/plugins/Degree/libDegreePlugin.so   ->  ../../../degree-rs/target/release/libdegree_rs.so
PluMA/plugins/Degree/Cargo.toml           ->  ../../../degree-rs/Cargo.toml
```

A minimal `config.txt`:

```text
Prefix example/
Plugin Degree  inputfile corrP.never.csv  outputfile /tmp/degree.out
```

PluMA's loader resolves `Degree_plugin_create / _input / _run / _output` via
`dlsym` and drives the standard input → run → output lifecycle. The output
prefix is treated as a file path (not a directory) — the plugin writes the
ranked-centrality table directly to it.

## Expected output shape

```text
Name                        Centrality      Rank
F.Burkholderiaceae.02       19.594364       1
Microbacterium.01           19.537722       2
Chryseobacterium.01         19.458588       3
...
```

Tab-separated, sorted descending by centrality. The total degree centrality is
`Σ_j |w(i, j)|` — the sum of absolute edge weights incident to node `i` —
matching the upstream C++ implementation.
