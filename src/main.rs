use clap::Parser;
use image::{Rgb, RgbImage};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of biomes to include. Format: Name or Name#RRGGBB.
    #[arg(short, long, num_args = 1..)]
    biomes: Option<Vec<String>>,

    /// Use black, white, and grey preset with default colors.
    #[arg(short, long)]
    grey: bool,

    /// Output file name for the standard nested mode.
    #[arg(short, long, default_value = "output_biomes.json")]
    output: String,

    /// Output per-biome grid files instead of a single nested JSON.
    #[arg(short = 'r', long)]
    grid: bool,

    /// Tile size in px, format: WxH (e.g. 32x32, 40x50).
    #[arg(short = 's', long, default_value = "32x32")]
    tile_size: String,

    /// Skip generating tileset images.
    #[arg(long)]
    no_image: bool,

    /// Number of base (blank) tiles to include at the beginning of each biome.
    #[arg(short = 'v', long, default_value = "1")]
    base_tiles: u32,
}

#[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Hash)]
struct Biome {
    name: String,
    display_name: String,
    color: [u8; 3],
}

impl Biome {
    fn new(input: &str) -> Self {
        let parts: Vec<&str> = input.split('#').collect();
        let name = parts[0].to_string();
        let name_lower = name.to_lowercase();
        
        let display_name = match name_lower.as_str() {
            "black" => "black_off".to_string(),
            "white" => "white_on".to_string(),
            "grey" => "grey_other".to_string(),
            _ => name_lower.clone(),
        };

        let color = if parts.len() > 1 {
            parse_hex_color(parts[1]).unwrap_or_else(|_| default_color(&name_lower))
        } else {
            default_color(&name_lower)
        };

        Self {
            name,
            display_name,
            color,
        }
    }

    fn as_str(&self) -> &str {
        &self.display_name
    }
}

fn parse_hex_color(hex: &str) -> Result<[u8; 3], String> {
    if hex.len() != 6 {
        return Err("Hex color must be 6 characters".to_string());
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
    Ok([r, g, b])
}

fn default_color(name: &str) -> [u8; 3] {
    match name {
        "black" => [0, 0, 0],
        "white" => [255, 255, 255],
        "grey" => [128, 128, 128],
        _ => [255, 0, 255], // Magenta for unknown
    }
}

#[derive(PartialOrd, Debug, Clone, Ord, Eq, PartialEq, Hash)]
struct BiomeEntry {
    pub here: Biome,
    pub north: Option<Biome>,
    pub east: Option<Biome>,
    pub south: Option<Biome>,
    pub west: Option<Biome>,
}

fn parse_tile_size(s: &str) -> (u32, u32) {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse::<u32>().unwrap_or(32);
        let h = parts[1].parse::<u32>().unwrap_or(32);
        (w, h)
    } else {
        (32, 32)
    }
}

fn main() {
    let args = Args::parse();
    let (tw, th) = parse_tile_size(&args.tile_size);

    let biome_inputs = if let Some(custom) = args.biomes {
        custom
    } else if args.grey {
        vec!["Black".to_string(), "White".to_string(), "Grey".to_string()]
    } else {
        vec!["Black".to_string(), "White".to_string()]
    };

    let biomes: Vec<Biome> = biome_inputs.iter().map(|s| Biome::new(s)).collect();

    let mut combos_per_biome: BTreeMap<Biome, BTreeSet<BiomeEntry>> = BTreeMap::new();
    for source in &biomes {
        for north in &biomes {
            for east in &biomes {
                for south in &biomes {
                    for west in &biomes {
                        let entry = BiomeEntry {
                            here: source.clone(),
                            north: Some(north.clone()).filter(|n| n != source),
                            east: Some(east.clone()).filter(|n| n != source),
                            south: Some(south.clone()).filter(|n| n != source),
                            west: Some(west.clone()).filter(|n| n != source),
                        };
                        combos_per_biome.entry(source.clone()).or_default().insert(entry);
                    }
                }
            }
        }
    }

    let mut tiles_per_biome: BTreeMap<Biome, Vec<BiomeEntry>> = BTreeMap::new();
    for (source, set) in combos_per_biome {
        let list: Vec<BiomeEntry> = set.into_iter().collect();
        // Since None < Some, the blank tile (all neighbors None) will be the first.
        let blank_tile = list[0].clone();
        
        let mut final_list = Vec::new();
        for _ in 1..args.base_tiles {
            final_list.push(blank_tile.clone());
        }
        final_list.extend(list);
        tiles_per_biome.insert(source, final_list);
    }

    let row_col_dim = biomes.len().pow(2);

    if args.grid {
        let other_biomes: Vec<String> = biomes.iter().map(|b| b.as_str().to_string()).collect();

        for (source, list) in &tiles_per_biome {
            let mut rows = Vec::new();
            let mut current_row_cells = Vec::new();
            let mut row_idx = 0;

            for (i, entry) in list.iter().enumerate() {
                let col_idx = i % row_col_dim;
                let cell = serde_json::json!({
                    "col": col_idx,
                    "north": entry.north.as_ref().map(|b| b.as_str()).unwrap_or("none"),
                    "east": entry.east.as_ref().map(|b| b.as_str()).unwrap_or("none"),
                    "south": entry.south.as_ref().map(|b| b.as_str()).unwrap_or("none"),
                    "west": entry.west.as_ref().map(|b| b.as_str()).unwrap_or("none"),
                });
                current_row_cells.push(cell);

                if (i + 1) % row_col_dim == 0 {
                    rows.push(serde_json::json!({
                        "row": row_idx,
                        "tiles": current_row_cells
                    }));
                    current_row_cells = Vec::new();
                    row_idx += 1;
                }
            }

            // Handle the last row if it's incomplete
            if !current_row_cells.is_empty() {
                rows.push(serde_json::json!({
                    "row": row_idx,
                    "tiles": current_row_cells
                }));
            }

            let num_rows = (list.len() + row_col_dim - 1) / row_col_dim;
            let output_data = serde_json::json!({
                "biome": source.as_str(),
                "total_tiles": list.len(),
                "other_biomes": other_biomes,
                "grid_shape": {
                    "width": row_col_dim,
                    "height": num_rows
                },
                "rows": rows
            });

            let filename = format!("{}_grid.json", source.as_str());
            let fw = File::create(&filename).unwrap();
            serde_json::to_writer_pretty(fw, &output_data).unwrap();
            println!("Successfully wrote grid to {}", filename);
        }
    } else {
        let mut json_root = serde_json::Value::Object(serde_json::Map::new());
        for (_source, list) in &tiles_per_biome {
            for (i, entry) in list.iter().enumerate() {
                let x = i % row_col_dim;
                let y = i / row_col_dim;
                update_json(&mut json_root, x, y, entry);
            }
        }

        let fw = File::create(&args.output).unwrap();
        serde_json::to_writer_pretty(fw, &json_root).unwrap();
        println!("Successfully wrote nested JSON to {}", args.output);
    }

    if !args.no_image {
        for (source, list) in &tiles_per_biome {
            let num_rows = (list.len() + row_col_dim - 1) / row_col_dim;
            let img_w = row_col_dim as u32 * tw;
            let img_h = num_rows as u32 * th;
            let mut img = RgbImage::new(img_w, img_h);

            let margin_w = (tw as f32 * 0.3) as u32;
            let margin_h = (th as f32 * 0.3) as u32;

            for (i, entry) in list.iter().enumerate() {
                let tx = (i % row_col_dim) as u32 * tw;
                let ty = (i / row_col_dim) as u32 * th;

                let here_rgb = Rgb(entry.here.color);
                let north_rgb = entry.north.as_ref().map(|b| Rgb(b.color)).unwrap_or(here_rgb);
                let east_rgb = entry.east.as_ref().map(|b| Rgb(b.color)).unwrap_or(here_rgb);
                let south_rgb = entry.south.as_ref().map(|b| Rgb(b.color)).unwrap_or(here_rgb);
                let west_rgb = entry.west.as_ref().map(|b| Rgb(b.color)).unwrap_or(here_rgb);

                for y in 0..th {
                    for x in 0..tw {
                        let px = tx + x;
                        let py = ty + y;
                        
                        let color = if y < margin_h {
                            north_rgb
                        } else if y >= th - margin_h {
                            south_rgb
                        } else if x < margin_w {
                            west_rgb
                        } else if x >= tw - margin_w {
                            east_rgb
                        } else {
                            here_rgb
                        };
                        
                        img.put_pixel(px, py, color);
                    }
                }
            }

            let filename = format!("{}_tileset.png", source.as_str());
            img.save(&filename).unwrap();
            println!("Successfully generated tileset image: {}", filename);
        }
    }
}

fn update_json(json_root: &mut Value, x: usize, y: usize, entry: &BiomeEntry) {
    json_root
        .as_object_mut()
        .unwrap()
        .entry(entry.here.as_str())
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry("north")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.north.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry("east")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.east.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry("south")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.south.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry("west")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.west.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        .or_insert(serde_json::json!({
            "row": y,
            "col": x,
            "file": entry.here.as_str(),
        }));
}
