#[derive(Debug, Clone)]
pub struct Chunk {
    blocks: Vec<Block>,
    dirty: bool,
    coord: ChunkCoord,
}

impl Chunk {
    pub const CHUNK_SIZE: usize = 16;

    pub fn empty(coord: ChunkCoord) -> Self {
        let blocks = vec![Block::default(); Self::CHUNK_SIZE * Self::CHUNK_SIZE * Self::CHUNK_SIZE];
        Self {
            blocks,
            dirty: false,
            coord
        }
    }

    pub fn test_chunk(coord: ChunkCoord) -> Self {
        let mut chunk = Self::empty(coord);
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

        if x >= Self::CHUNK_SIZE || z >= Self::CHUNK_SIZE || y >= Self::CHUNK_SIZE {
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

    pub fn aabb(&self) -> AABB {
        self.coord.aabb()
    }

    fn block_index(index: LocalCoord) -> usize {
        let (x, y, z) = (index.x, index.y, index.z);
        x + z * Self::CHUNK_SIZE + y * Self::CHUNK_SIZE * Self::CHUNK_SIZE
    }
}

pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}


#[derive(Copy, Clone)]
pub struct BlockTextures {
    pub top: u32,
    pub bottom: u32,
    pub north: u32,
    pub south: u32,
    pub east: u32,
    pub west: u32,
}


#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Block {
    #[default]
    Air,
    Dirt,
    Water,
    Stone
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

    pub fn aabb(&self) -> AABB {
        let min = glam::Vec3::new(
            (self.x * Chunk::CHUNK_SIZE as isize) as f32,
            (self.y * Chunk::CHUNK_SIZE as isize) as f32,
            (self.z * Chunk::CHUNK_SIZE as isize) as f32,
        );

        let max = glam::Vec3::new(
            min.x + Chunk::CHUNK_SIZE as f32,
            min.y + Chunk::CHUNK_SIZE as f32,
            min.z + Chunk::CHUNK_SIZE as f32,
        );

        AABB { min, max }
    }
}

impl WorldCoord {
    pub fn to_chunk(&self) -> (ChunkCoord, LocalCoord) {
        let cx = self.x.div_euclid(Chunk::CHUNK_SIZE as isize);
        let cy = self.y.div_euclid(Chunk::CHUNK_SIZE as isize);
        let cz = self.z.div_euclid(Chunk::CHUNK_SIZE as isize);

        let bx = self.x.rem_euclid(Chunk::CHUNK_SIZE as isize) as usize;
        let by = self.y.rem_euclid(Chunk::CHUNK_SIZE as isize) as usize;
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