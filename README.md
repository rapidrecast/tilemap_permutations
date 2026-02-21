# Tilemap Permutations Generator

A CLI tool designed to generate a comprehensive JSON mapping of all possible biome permutations for tile-based mapping systems. This is particularly useful for auto-tiling systems where you need to know which tile (indexed by row and column) corresponds to a specific combination of neighboring biomes.

## Features

- **Nested Permutations**: Generates a self-documenting JSON structure following the path: `[biome of interest] -> north -> [north_biome] -> east -> [east_biome] -> south -> [south_biome] -> west -> [west_biome]`.
- **Custom Biomes**: Provide any number of biome names to generate permutations for.
- **Presets**: Quick flags for common biome sets (Black/White and Black/White/Grey).
- **Coordinate Indexing**: Automatically assigns `row` and `col` indices for each unique combination, assuming a square grid layout for each source biome's tileset.

## Installation

To run this tool, you need the Rust toolchain installed on your system.

### Install Rust (via rustup)

If you don't have Rust installed, the recommended way is using `rustup`:

1.  **On macOS or Linux**:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2.  **On Windows**:
    Download and run [rustup-init.exe](https://rustup.rs/).

Follow the on-screen instructions. Once finished, restart your terminal and verify the installation:
```bash
rustc --version
```

## How to Run

Clone the repository and run using `cargo`:

### Default (Black and White)
By default, the tool generates permutations for "Black" and "White" biomes.
```bash
cargo run
```

### Black, White, and Grey
Use the `--grey` (or `-g`) flag to include a "Grey" biome.
```bash
cargo run -- --grey
```

### Custom Biomes
You can provide any number of biomes using the `--biomes` (or `-b`) flag.
```bash
cargo run -- --biomes Forest Desert Tundra Water
```

### Custom Output File
Specify a different output filename with `--output` (or `-o`).
```bash
cargo run -- --biomes Ash Lava -o volcanic_map.json
```

## Using the Output

The tool generates a JSON file (default: `output_biomes.json`). The structure is deeply nested to allow for easy traversal in game engines or tiling scripts:

```json
{
  "black_off": {
    "north": {
      "white_on": {
        "east": {
          "none": {
            "south": {
              "white_on": {
                "west": {
                  "none": {
                    "row": 0,
                    "col": 2,
                    "file": "black_off"
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
```

- **`row` / `col`**: The calculated position of the tile in a sprite sheet.
- **`file`**: The source biome identifier.
- **`none`**: Indicates that the neighbor in that direction is the same as the "here" biome (no transition).

This allows your game logic to determine the correct tile index by simply traversing the JSON object based on the surrounding biomes.
