"""Generate a synthetic weighted-adjacency CSV for the Degree plugin example.

The PluMA Degree plugin reads a square `n × n` matrix where row `i` and column
`j` carry the signed weight of the edge between nodes `i` and `j`. We
simulate the corncob/SparCC-style sparse signed correlation network that the
upstream Degree plugin was designed to consume:

  * 40 OTUs grouped into 4 modules of 10 each.
  * Within-module edges are positive (correlation ~0.7-0.95).
  * A handful of cross-module antagonistic edges (negative ~-0.7-0.9).
  * The rest are zero (sparse).

Seed is fixed at 42 so the output is bit-identical on every run.

Outputs (committed):

  * `network.csv` — the 40 × 40 signed adjacency matrix.
  * `expected_output.txt` — reference centrality table, written by running
    the Rust plugin once after CSV generation. The example test diffs
    against this file.

Run:

    python example/generate_test_data.py
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).parent
ROOT = HERE.parent

N_MODULES = 4
PER_MODULE = 10
N = N_MODULES * PER_MODULE
SEED = 42


def generate_matrix() -> np.ndarray:
    rng = np.random.default_rng(SEED)
    W = np.zeros((N, N), dtype=float)
    # Within-module: positive correlations, ~70% present.
    for m in range(N_MODULES):
        base = m * PER_MODULE
        for i in range(PER_MODULE):
            for j in range(i + 1, PER_MODULE):
                if rng.random() < 0.7:
                    w = rng.uniform(0.7, 0.95)
                    W[base + i, base + j] = w
                    W[base + j, base + i] = w
    # Cross-module antagonistic edges: ~3 per module pair.
    for a in range(N_MODULES):
        for b in range(a + 1, N_MODULES):
            for _ in range(3):
                i = rng.integers(0, PER_MODULE)
                j = rng.integers(0, PER_MODULE)
                w = -rng.uniform(0.7, 0.9)
                W[a * PER_MODULE + i, b * PER_MODULE + j] = w
                W[b * PER_MODULE + j, a * PER_MODULE + i] = w
    # Self-loops = 1, matching the upstream Degree fixture (corrP.never.csv).
    np.fill_diagonal(W, 1.0)
    return W


def write_csv(W: np.ndarray, path: Path) -> None:
    labels = [f"OTU{i:02d}" for i in range(N)]
    with path.open("w") as fh:
        # First row: empty cell + node labels (matches corrP.never.csv shape).
        fh.write('""' + "," + ",".join(f'"{l}"' for l in labels) + "\n")
        for i, label in enumerate(labels):
            fh.write(f'"{label}"')
            for j in range(N):
                fh.write(f",{W[i, j]:.6g}")
            fh.write("\n")


def main() -> int:
    csv_path = HERE / "network.csv"
    W = generate_matrix()
    write_csv(W, csv_path)
    print(f"wrote {csv_path}  ({N} × {N}, {int((W != 0).sum())} non-zero entries)")

    # Build the release binary and run it once on the new CSV to produce
    # the reference output the example pipeline asserts against.
    out_path = HERE / "expected_output.txt"
    print(f"building release binary...", flush=True)
    subprocess.run(
        ["cargo", "build", "--release", "--bin", "degree"],
        cwd=ROOT, check=True,
    )
    print(f"running degree binary -> {out_path}", flush=True)
    subprocess.run(
        [str(ROOT / "target" / "release" / "degree"), str(csv_path), str(out_path)],
        check=True,
    )
    print(f"wrote {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
