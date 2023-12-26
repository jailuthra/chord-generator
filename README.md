# Chord Generator

This is for a project inspired by Guitarmadillo's chord helper https://guitarmadillo.com/guitar-chords

## How to use?

Build a non-debug binary by running:

```bash
cargo build --release
```

Then run the binary, and save the JSON output

```bash
target/release/chord-generator > out.json
```

It will generate multiple guitar fingerings for all possible chords for
standard EADGBE tuning.

To generate chords for a different tuning, you can edit the `DEFAULT_TUNING`
array in `main.rs` and build + run again.
