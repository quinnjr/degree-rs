# degree-rs example

Self-contained, fully-deterministic fixture for the Degree centrality plugin
that runs through a real `pluma` binary end-to-end. Synthetic 40-node signed
correlation network: 4 modules of 10 OTUs each with within-module positive
edges (`+0.7..+0.95`) and cross-module antagonistic edges (`-0.7..-0.9`), the
sparse signed-correlation pattern Degree was designed for.

## Files

| File | Shape | Contents |
|---|---|---|
| `network.csv` | 40 × 40 | Synthetic signed adjacency matrix. First row + first column are OTU labels; cell `(i, j)` is the signed edge weight (or 0 for absent edges). |
| `expected_output.txt` | 40 × 3 | Reference centrality table, written by the Rust plugin at fixture-generation time. The pipeline diffs the live output against this. |
| `config.txt` | — | Minimal PluMA pipeline (`Prefix example/` + one `Plugin` line). |
| `run_pluma_example.sh` | — | One-shot wrapper: builds the `.so`, stages it under a tmp `PLUMA_PLUGIN_PATH`, invokes pluma, diffs the output against `expected_output.txt`. |
| `generate_test_data.py` | — | Deterministic regenerator for `network.csv` + `expected_output.txt` (seed 42). |
| `corrP.never.csv`, `output.txt` | 126 × 126 | Bonus real-world fixture (oral-microbiome corncob/SparCC network from the upstream `movingpictures83/Degree` tests) — kept for legacy comparisons; not used by the pluma pipeline. |

## Running through the pluma binary

```bash
./example/run_pluma_example.sh                    # uses `pluma` from $PATH
./example/run_pluma_example.sh /path/to/pluma     # explicit binary
PLUMA=/path/to/pluma ./example/run_pluma_example.sh
```

The script builds the release `.so`, stages it under a temporary
`PLUMA_PLUGIN_PATH` as `Degree/libDegreePlugin.so`, runs
`pluma example/config.txt` from the repo root, and diffs `example/out/degree.txt`
against the committed `expected_output.txt`. Returns non-zero on mismatch.

Requires a `pluma` built with `--with-rust` and the `HAVE_RUST`-define fix
from [FIUBioRG/PluMA#13](https://github.com/FIUBioRG/PluMA/pull/13) (without
the fix the Rust loader's body is `#ifdef`-stripped and Degree silently
won't run).

## Regenerating the synthetic data

```bash
python example/generate_test_data.py
```

Idempotent — same seed yields the same bytes every run. The generator also
shells out to `cargo build --release && target/release/degree network.csv
expected_output.txt` to refresh the reference output so it stays in sync
with whatever the current code emits.

## Expected output shape

Tab-separated, sorted descending by centrality:

```text
Name      Centrality   Rank
OTU09     8.295446     1
OTU31     7.613305     2
OTU24     7.584769     3
...
```

The total degree centrality is `Σⱼ |w(i, j)|` — the sum of absolute edge
weights incident to node `i` — matching the upstream C++ implementation.
