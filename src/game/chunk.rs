#[derive(Debug, Clone)]
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
                let coord1 = LocalCoord { x, y: 0, z };
                let coord2 = LocalCoord { x: 0, y: x, z };
                chunk.set_block(coord1, Block::Dirt);
                chunk.set_block(coord2, Block::Dirt);
            }
        }

        chunk
    }

    pub fn set_block(&mut self, index: LocalCoord, block: Block) {
        let i = Self::block_index(index);
        let current = self.blocks[i];

        if block != current {
            self.dirty = true;
            self.blocks[i] = block;
        }
    }

    pub fn get_block(&self, index: LocalCoord) -> Block {
        let (x, y, z) = (index.x, index.y, index.z);

        if x >= Self::CHUNK_SIZE || z >= Self::CHUNK_SIZE || y >= Self::CHUNK_HEIGHT {
            return Block::Air;
        }

        self.blocks[Self::block_index(index)]
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn block_index(index: LocalCoord) -> usize {
        let (x, y, z) = (index.x, index.y, index.z);
        x + z * Self::CHUNK_SIZE + y * Self::CHUNK_SIZE * Self::CHUNK_SIZE
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
            Block::Dirt => Tile { x: 7, y: 5 },
            Block::Water => Tile { x: 7, y: 9 },
            Block::Stone => Tile { x: 2, y: 4 },
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ChunkCoord {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct WorldCoord {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct LocalCoord {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl From<(isize, isize, isize)> for ChunkCoord {
    fn from(value: (isize, isize, isize)) -> Self {
        let (x, y, z) = value;
        Self { x, y, z }
    }
}

impl From<(isize, isize, isize)> for WorldCoord {
    fn from(value: (isize, isize, isize)) -> Self {
        let (x, y, z) = value;
        Self { x, y, z }
    }
}

impl From<(usize, usize, usize)> for LocalCoord {
    fn from(value: (usize, usize, usize)) -> Self {
        let (x, y, z) = value;
        Self { x, y, z }
    }
}

impl ChunkCoord {
    pub fn as_tuple(&self) -> (isize, isize, isize) {
        return (self.x, self.y, self.z);
    }
}

impl WorldCoord {
    pub fn to_chunk(&self) -> (ChunkCoord, LocalCoord) {
        let cx = self.x.div_euclid(Chunk::CHUNK_SIZE as isize);
        let cy = self.y.div_euclid(Chunk::CHUNK_HEIGHT as isize);
        let cz = self.z.div_euclid(Chunk::CHUNK_SIZE as isize);

        let bx = self.x.rem_euclid(Chunk::CHUNK_SIZE as isize) as usize;
        let by = self.y.rem_euclid(Chunk::CHUNK_HEIGHT as isize) as usize;
        let bz = self.z.rem_euclid(Chunk::CHUNK_SIZE as isize) as usize;

        (
            ChunkCoord { x: cx, y: cy, z: cz },
            LocalCoord { x: bx, y: by, z: bz },
        )
    }
    pub fn as_tuple(&self) -> (isize, isize, isize) {
        return (self.x, self.y, self.z);
    }
}

impl LocalCoord {
    pub fn as_tuple(&self) -> (usize, usize, usize) {
        return (self.x, self.y, self.z);
    }
}