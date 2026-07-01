# Performance benchmarks

robost's key vision differentiator is DPI-resilient template matching: instead of
requiring a pixel-exact template, it retries at multiple scales (`1.0`, `0.8`, `1.25`
by default) so a UI element captured at one DPI setting is still found when the
screen renders at another. That resilience isn't free, so this page quantifies its
cost with real numbers instead of leaving the claim unbacked.

## What's benchmarked

`crates/robost-vision/benches/template_match.rs` (criterion, `cargo bench -p
robost-vision`) covers three scenarios against a synthetically generated 480x270
haystack (no binary image fixtures needed — same generation approach as
`examples/find_all_example.rs`):

- **`find_single_scale`** — the template appears at its native size; single-scale
  matching finds it in one NCC pass.
- **`find_multi_scale`** — the template appears at 80% of its native size (a
  DPI-mismatch scenario). The scale-1.0 attempt is forced to fail (via a strict
  threshold) so the matcher genuinely falls through to the 0.8 attempt, rather than
  accidentally succeeding on the first try.
- **`find_all_multiple_matches`** — five occurrences of the template in one
  haystack, exercising `find_all`'s full-image scan plus its O(n²) non-max
  suppression pass across candidates.

## Results

Measured locally on an Apple M4 (macOS 26.5.1), `cargo bench -p robost-vision`,
20 samples per benchmark:

| Benchmark | Mean time | What it shows |
|---|---|---|
| `find_single_scale` | ~196 ms | Baseline: one NCC pass over a 480x270 haystack with a 40x40 template |
| `find_multi_scale` | ~312 ms | ~1.6x the single-scale baseline — the real cost of falling through to a second scale attempt to resolve a DPI mismatch |
| `find_all_multiple_matches` | ~680 ms | Full-image multi-scale scan across 5 occurrences plus non-max suppression |

These are absolute numbers on one machine and will vary with hardware, haystack
size, and template complexity — treat them as a reproducible reference point, not
a guaranteed SLA. Re-run `cargo bench -p robost-vision` locally to get numbers for
your own machine and workload.

## Why no RDP/VNC-specific benchmark

robost's agentless RDP/Citrix/VNC automation isn't implemented as a separate,
protocol-aware code path — it works by capturing whatever window is on screen
locally (the RDP/VNC client included) and running the same generic
capture-and-match pipeline benchmarked above. There's no dedicated network or
protocol logic in this codebase to benchmark in isolation; "no agent required on
the remote machine" is an architectural property, not a runtime performance
characteristic, so it isn't something a benchmark can quantify.
