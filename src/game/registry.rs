use std::collections::HashMap;

use serde::Deserialize;

use crate::{game::chunk::{Block, BlockTextures}, render::assets::Assets};

pub struct BlockRegistry {
    definitions: Vec<BlockDefinition>,
    unique_paths: Vec<String>,
}

impl BlockRegistry {
    pub fn from_str(s: &str, assets: &Assets) -> Self {
        let config = toml::from_str::<Configuration>(s).unwrap();
        Self::from_config(config, assets)
    }

    pub fn from_config(config: Configuration, assets: &Assets) -> Self {
        let mut texture_indices = HashMap::new();
        let mut unique_paths = Vec::new();

        let mut register = |path: &str| -> u32 {
            if let Some(&idx) = texture_indices.get(path) {
                idx
            } else {
                let idx = unique_paths.len() as u32;
                unique_paths.push(path.to_string());
                texture_indices.insert(path.to_string(), idx);
                idx
            }
        };

        let mut definitions = vec![BlockDefinition {
                textures: BlockTextures { top: 0, bottom: 0, north: 0, south: 0, east: 0, west: 0 },
                solid: false,
                opaque: false,
        }; config.blocks.len() + 1];

        for block in config.blocks {
            let textures = BlockTextures {
                top: register(&block.top),
                bottom: register(&block.bottom),
                north: register(&block.north),
                south: register(&block.south),
                east: register(&block.east),
                west: register(&block.west),
            };

            let def = BlockDefinition {
                textures,
                solid: block.solid,
                opaque: block.opaque
            };

            let block_type = match block.name.as_str() {
                "Dirt" => Block::Dirt,
                "Stone" => Block::Stone,
                "Water" => Block::Water,
                other => panic!("Invalid block name provided. Please modify registry.")
            };

            definitions[block_type as usize] = def;
        }


        Self {
            definitions,
            unique_paths,
        }
    }

    pub fn get(&self, block: Block) -> &BlockDefinition {
        &self.definitions[block as usize]
    }

    pub fn unique_paths(&self) -> &Vec<String> {
        &self.unique_paths
    }
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    blocks: Vec<BlockConfig>
}

#[derive(Debug, Deserialize)]
pub struct BlockConfig {
    name: String,
    top: String,
    bottom: String,
    east: String,
    west: String,
    north: String,
    south: String,
    opaque: bool,
    solid: bool,
}

#[derive(Clone)]
pub struct BlockDefinition {
    pub textures: BlockTextures,
    pub solid: bool,
    pub opaque: bool,
}