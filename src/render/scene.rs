use crate::{game::chunk::ChunkCoord, render::mesh::CpuMesh};

pub struct RenderScene {
    pub dirty_chunks: Vec<(ChunkCoord, CpuMesh)>,
}