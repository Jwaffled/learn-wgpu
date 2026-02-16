use libnoise::{Generator, Perlin, Scale, Source};

use crate::game::chunk::{Block, Chunk, ChunkCoord, LocalCoord};

#[derive(Clone)]
pub struct WorldGenerator {
    generator: Scale<2, Perlin<2>>,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        let generator = Source::perlin(seed).scale([0.1; 2]);
        Self {
            generator
        }
    }

    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        println!("Generating chunk @ {:?}", coord);
        let mut chunk = Chunk::empty();
        for x in 0..Chunk::CHUNK_SIZE {
            for z in 0..Chunk::CHUNK_SIZE {
                let world_x = coord.x * Chunk::CHUNK_SIZE as isize + x as isize;
                let world_z = coord.z * Chunk::CHUNK_SIZE as isize + z as isize;

                let height = self.height_at(world_x, world_z);
                for y in 0..Chunk::CHUNK_HEIGHT {
                    if y as isize == height {
                        chunk.set_block(
                            LocalCoord { x, y, z },
                            Block::Water,
                        );
                    } else if (y as isize) < height {
                        chunk.set_block(
                            LocalCoord { x, y, z },
                            Block::Stone,
                        );
                    }
                }
            }
        }

        chunk
    }

    fn height_at(&self, x: isize, z: isize) -> isize {
        let n = self.generator.sample([x as f64, z as f64]);
        ((n + 1.0) * 0.5 * 20.0) as isize + 20
    }
}