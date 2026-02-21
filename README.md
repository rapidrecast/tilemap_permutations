# Tilemap Permutations Generator

A CLI tool designed to generate a comprehensive JSON mapping of all possible biome permutations for tile-based mapping systems. This is particularly useful for auto-tiling systems where you need to know which tile (indexed by row and column) corresponds to a specific combination of neighboring biomes.

## Features

- **Nested Permutations**: Generates a self-documenting JSON structure following the path: `[biome of interest] -> north -> [north_biome] -> east -> [east_biome] -> south -> [south_biome] -> west -> [west_biome]`.
- **Grid Output Mode**: Generates a separate file per biome with a structured row-by-row layout, including coordinate metadata and a summary of biomes and dimensions.
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

### Standard Nested Mode (Default)
Generates a single `output_biomes.json` file.
```bash
cargo run
```

### Grid Mode
Generates one file per biome (e.g., `black_off_grid.json`) with a structured 2D layout.
```bash
cargo run -- --grid # or -r
```

### Black, White, and Grey Preset
```bash
cargo run -- --grey # or -g
```

### Custom Biomes
```bash
cargo run -- --biomes Forest Desert Tundra Water
```

### Custom Output File (Nested Mode)
```bash
cargo run -- --biomes Ash Lava -o volcanic_map.json
```

## Using the Output

### Nested Mode (Default)
The structure is deeply nested for easy traversal by neighbor state:
`root.biome.north.biome.east.biome.south.biome.west.biome.property`

### Grid Mode (`--grid`)
Generates files like `black_off_grid.json`. This mode provides a row-based layout with metadata:

```json
{
  "biome": "black_off",
  "total_tiles": 16,
  "other_biomes": ["black_off", "white_on"],
  "grid_shape": { "width": 4, "height": 4 },
  "rows": [
    {
      "row": 0,
      "tiles": [
        { "col": 0, "north": "none", "east": "none", "south": "none", "west": "none" },
        { "col": 1, "north": "none", "east": "none", "south": "none", "west": "white_on" }
      ]
    }
  ]
}
```

- **`row` / `col`**: The logical coordinates within the tileset.
- **`grid_shape`**: The width and height of the tileset grid.
- **`none`**: Indicates that the neighbor in that direction is the same as the current biome.
- **biome_name**: Indicates a transition to that specific biome.
