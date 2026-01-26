use crate::{game::chunk::{Block, Chunk}, render::{mesh::CpuMesh, vertex::Vertex}};

pub struct ChunkMesher {

}

struct Face {
    normal: [f32; 3],
    corners: [[f32; 3]; 4],
    neighbor_offset: (isize, isize, isize)
}

const FACES: [Face; 6] = [
    // +X
    Face {
        normal: [1.0, 0.0, 0.0],
        neighbor_offset: (1, 0, 0),
        corners: [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
    },
    // -X
    Face {
        normal: [-1.0, 0.0, 0.0],
        neighbor_offset: (-1, 0, 0),
        corners: [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
    },
    // +Y
    Face {
        normal: [0.0, 1.0, 0.0],
        neighbor_offset: (0, 1, 0),
        corners: [
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    },
    // -Y
    Face {
        normal: [0.0, -1.0, 0.0],
        neighbor_offset: (0, -1, 0),
        corners: [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    },
    // +Z
    Face {
        normal: [0.0, 0.0, 1.0],
        neighbor_offset: (0, 0, 1),
        corners: [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
    },
    // -Z
    Face {
        normal: [0.0, 0.0, -1.0],
        neighbor_offset: (0, 0, -1),
        corners: [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
    },
];

impl ChunkMesher {
    pub fn create_mesh(chunk: &Chunk, chunk_coord: (usize, usize, usize)) -> CpuMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let (chunk_x, chunk_y, chunk_z) = chunk_coord;
        let (chunk_offset_x, chunk_offset_y, chunk_offset_z) = (chunk_x * Chunk::CHUNK_SIZE, chunk_y * Chunk::CHUNK_HEIGHT, chunk_z * Chunk::CHUNK_SIZE);

        for y in 0..Chunk::CHUNK_HEIGHT {
            for z in 0..Chunk::CHUNK_SIZE {
                for x in 0..Chunk::CHUNK_SIZE {
                    let block = chunk[(x, y, z)];
                    if block == Block::Air {
                        continue;
                    }

                    for face in &FACES {
                        let (dx, dy, dz) = face.neighbor_offset;
                        let neighbor = chunk.get_block((
                            x as isize + dx,
                            y as isize + dy,
                            z as isize + dz
                        ));

                        if neighbor != Block::Air {
                            continue;
                        }

                        let base_index = vertices.len() as u32;

                        let (uv_min, uv_max) = block.get_tile().uv_rect();

                        let uvs = [
                            [uv_min[0], uv_min[1]],
                            [uv_max[0], uv_min[1]],
                            [uv_max[0], uv_max[1]],
                            [uv_min[0], uv_max[1]]
                        ];

                        for (i, corner) in face.corners.iter().enumerate() {
                            
                            vertices.push(Vertex {
                                position: [
                                    (x + chunk_offset_x) as f32 + corner[0],
                                    (y + chunk_offset_y) as f32 + corner[1],
                                    (z + chunk_offset_z) as f32 + corner[2],
                                ],
                                tex_coords: uvs[i],
                                normal: face.normal
                            });
                        }

                        indices.extend_from_slice(&[
                            base_index,
                            base_index + 1,
                            base_index + 2,
                            base_index,
                            base_index + 2,
                            base_index + 3,
                        ]);
                    }
                }
            }
        }

        CpuMesh {
            vertices,
            indices
        }
    }
}

