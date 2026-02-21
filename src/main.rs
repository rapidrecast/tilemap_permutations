use clap::Parser;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of biomes to include.
    #[arg(short, long, num_args = 1..)]
    biomes: Option<Vec<String>>,

    /// Use black, white, and grey preset.
    #[arg(short, long)]
    grey: bool,

    /// Output file name.
    #[arg(short, long, default_value = "output_biomes.json")]
    output: String,
}

#[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Hash)]
struct Biome {
    display_name: String,
}

impl Biome {
    fn new(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        let display_name = match name_lower.as_str() {
            "black" => "black_off".to_string(),
            "white" => "white_on".to_string(),
            "grey" => "grey_other".to_string(),
            _ => name_lower,
        };
        Self { display_name }
    }

    fn as_str(&self) -> &str {
        &self.display_name
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

fn main() {
    let args = Args::parse();

    let biome_names = if let Some(custom) = args.biomes {
        custom
    } else if args.grey {
        vec!["Black".to_string(), "White".to_string(), "Grey".to_string()]
    } else {
        vec!["Black".to_string(), "White".to_string()]
    };

    let biomes: Vec<Biome> = biome_names.iter().map(|s| Biome::new(s)).collect();

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

    let mut json_root = serde_json::Value::Object(serde_json::Map::new());
    for (_source, set) in combos_per_biome {
        let row_col_dim = biomes.len().pow(2);
        for (i, entry) in set.iter().enumerate() {
            let x = i % row_col_dim;
            let y = i / row_col_dim;
            update_json(&mut json_root, x, y, entry);
        }
    }

    let fw = File::create(&args.output).unwrap();
    serde_json::to_writer_pretty(fw, &json_root).unwrap();
    println!("Successfully wrote to {}", args.output);
}

fn update_json(json_root: &mut Value, x: usize, y: usize, entry: &BiomeEntry) {
    json_root
        .as_object_mut()
        .unwrap()
        .entry(entry.here.as_str())
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        // Insert "north" object so it's self-documenting json
        .entry("north")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.north.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        // Insert "east object so it's self-documenting json
        .entry("east")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.east.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        // South
        .entry("south")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.south.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        // West
        .entry("west")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
        .unwrap()
        .entry(entry.west.as_ref().map(|b| b.as_str()).unwrap_or("none"))
        // At this point we know it's a unique combination.
        // We need to insert the index
        .or_insert(serde_json::json!({
            "row": y,
            "col": x,
            "file": entry.here.as_str(),
        }));
}
