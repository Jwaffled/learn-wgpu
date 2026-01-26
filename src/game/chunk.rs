use std::ops::Index;

pub struct Chunk {
    blocks: Vec<Block>,
    dirty: bool,
}

impl Chunk {
    pub const CHUNK_SIZE: usize = 16;
    pub const CHUNK_HEIGHT: usize = 64;

    pub fn empty() -> Self {
        let blocks = vec![Block::default(); Self::CHUNK_SIZE * Self::CHUNK_HEIGHT * Self::CHUNK_SIZE];
        Self {
            blocks,
            dirty: false,
        }
    }

    pub fn test_chunk() -> Self {
        let mut chunk = Self::empty();
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block((x, 0, z), Block::Dirt);
                chunk.set_block((0, x, z), Block::Dirt);
            }
        }

        chunk
    }

    pub fn set_block(&mut self, index: BlockCoord, block: Block) {
        let i = Self::block_index(index);
        let current = self.blocks[i];

        if block != current {
            self.dirty = true;
            self.blocks[i] = block;
        }
    }

    pub fn get_block(&self, index: (isize, isize, isize)) -> Block {
        let (x, y, z) = index;
        if x < 0 || y < 0 || z < 0 {
            return Block::Air;
        }

        let (x, y, z) = (x as usize, y as usize, z as usize);

        if x >= Self::CHUNK_SIZE || z >= Self::CHUNK_SIZE || y >= Self::CHUNK_HEIGHT {
            return Block::Air;
        }

        self[(x, y, z)]
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn block_index(index: BlockCoord) -> usize {
        let (x, y, z) = index;
        x + z * Self::CHUNK_SIZE + y * Self::CHUNK_SIZE * Self::CHUNK_SIZE
    }
}

impl Index<BlockCoord> for Chunk {
    type Output = Block;

    fn index(&self, index: ChunkCoord) -> &Self::Output {
        let i = Self::block_index(index);
        &self.blocks[i]
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Block {
    #[default]
    Air,
    Dirt,
    Water,
    Stone
}

impl Block {
    pub fn get_tile(&self) -> Tile {
        match *self {
            Block::Air => panic!("Tried to grab air texture; none exists"),
            Block::Dirt => Tile { x: 2, y: 0 },
            Block::Water => Tile { x: 7, y: 9 },
            Block::Stone => Tile { x: 3, y: 4 },
        }
    }
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
}

impl Tile {
    pub fn uv_rect(&self) -> ([f32; 2], [f32; 2]) {
        const ATLAS_WIDTH: f32 = 1152.0;
        const ATLAS_HEIGHT: f32 = 1280.0;
        const TILE_SIZE: f32 = 128.0;

        let u_min = self.x as f32 * TILE_SIZE / ATLAS_WIDTH;
        let v_min = self.y as f32 * TILE_SIZE / ATLAS_HEIGHT;

        let u_max = (self.x as f32 * TILE_SIZE + TILE_SIZE) / ATLAS_WIDTH;
        let v_max = (self.y as f32 * TILE_SIZE + TILE_SIZE) / ATLAS_HEIGHT;

        ([u_min, v_min], [u_max, v_max])
    }
}

pub type ChunkCoord = (usize, usize, usize);
pub type BlockCoord = (usize, usize, usize);