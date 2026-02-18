use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex, mpsc::{self, Receiver, Sender}}, thread, time::{Duration, Instant}};

use crate::{game::{chunk::{Block, Chunk, ChunkCoord, LocalCoord}, generator::WorldGenerator, player::Player, registry::BlockRegistry, world::{ChunkEvent, FrameEvent, WorldState}}, render::{mesh::CpuMesh, renderers::debug_text::ChunkStats, vertex::{QuadCorner, Vertex, VoxelFace}}};

pub struct ChunkMesher {

}

const QUAD_UVS: [[f32; 2]; 4] = [
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
    [0.0, 1.0],
];
struct Face {
    direction: VoxelFace,
    normal: [f32; 3],
    corners: [[f32; 3]; 4],
    corner_pos: [QuadCorner; 4],
    neighbor_offset: (isize, isize, isize),
    uv_indices: [usize; 4],
}

const FACES: [Face; 6] = [
    // +X (East)
    Face {
        direction: VoxelFace::East,
        normal: [1.0, 0.0, 0.0],
        neighbor_offset: (1, 0, 0),
        corners: [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
        corner_pos: [
            QuadCorner::BottomLeft,
            QuadCorner::TopLeft,
            QuadCorner::TopRight,
            QuadCorner::BottomRight,
        ],
        uv_indices: [1, 2, 3, 0],
    },

    // -X (West)
    Face {
        direction: VoxelFace::West,
        normal: [-1.0, 0.0, 0.0],
        neighbor_offset: (-1, 0, 0),
        corners: [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        corner_pos: [
            QuadCorner::BottomLeft,
            QuadCorner::TopLeft,
            QuadCorner::TopRight,
            QuadCorner::BottomRight,
        ],
        uv_indices: [0, 3, 2, 1],
    },

    // +Y (Top)
    Face {
        direction: VoxelFace::Top,
        normal: [0.0, 1.0, 0.0],
        neighbor_offset: (0, 1, 0),
        corners: [
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        corner_pos: [
            QuadCorner::BottomLeft,
            QuadCorner::BottomRight,
            QuadCorner::TopRight,
            QuadCorner::TopLeft,
        ],
        uv_indices: [0, 1, 2, 3],
    },

    // -Y (Bottom)
    Face {
        direction: VoxelFace::Bottom,
        normal: [0.0, -1.0, 0.0],
        neighbor_offset: (0, -1, 0),
        corners: [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
        corner_pos: [
            QuadCorner::BottomLeft,
            QuadCorner::BottomRight,
            QuadCorner::TopRight,
            QuadCorner::TopLeft,
        ],
        uv_indices: [3, 2, 1, 0],
    },

    // +Z (North)
    Face {
        direction: VoxelFace::North,
        normal: [0.0, 0.0, 1.0],
        neighbor_offset: (0, 0, 1),
        corners: [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        corner_pos: [
            QuadCorner::BottomLeft,
            QuadCorner::BottomRight,
            QuadCorner::TopRight,
            QuadCorner::TopLeft,
        ],
        uv_indices: [0, 1, 2, 3],
    },

    // -Z (South)
    Face {
        direction: VoxelFace::South,
        normal: [0.0, 0.0, -1.0],
        neighbor_offset: (0, 0, -1),
        corners: [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        corner_pos: [
            QuadCorner::BottomLeft,
            QuadCorner::BottomRight,
            QuadCorner::TopRight,
            QuadCorner::TopLeft,
        ],
        uv_indices: [1, 0, 3, 2],
    },
];


impl ChunkMesher {
    pub fn create_mesh(chunk: &Chunk, registry: &BlockRegistry) -> CpuMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for y in 0..Chunk::CHUNK_SIZE as isize {
            for z in 0..Chunk::CHUNK_SIZE as isize {
                for x in 0..Chunk::CHUNK_SIZE as isize {
                    let block = chunk.get_block(LocalCoord { x: x as usize, y: y as usize, z: z as usize });
                    if block == Block::Air {
                        continue;
                    }

                    for face in &FACES {
                        let (dx, dy, dz) = face.neighbor_offset;
                        
                        let neighbor = if x + dx < 0 || y + dy < 0 || z + dz < 0 {
                            Block::Air
                        } else {
                            let coord = LocalCoord {
                                x: (x + dx) as usize,
                                y: (y + dy) as usize,
                                z: (z + dz) as usize
                            };
                            chunk.get_block(coord)
                        };

                        if neighbor != Block::Air {
                            continue;
                        }

                        let base_index = vertices.len() as u32;

                        let def = registry.get(block);
                        let texture_index = match face.direction {
                            VoxelFace::Bottom => def.textures.bottom,
                            VoxelFace::East => def.textures.east,
                            VoxelFace::North => def.textures.north,
                            VoxelFace::South => def.textures.south,
                            VoxelFace::Top => def.textures.top,
                            VoxelFace::West => def.textures.west,
                        };

                        for (i, corner) in face.corners.iter().enumerate() {
                            let data = Vertex::pack_vertex(
                                x + corner[0] as isize,
                                y + corner[1] as isize,
                                z + corner[2] as isize,
                                face.corner_pos[i],
                                face.direction,
                                texture_index
                            );
                            vertices.push(Vertex { data });
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

pub struct ChunkManager {
    chunks: HashMap<ChunkCoord, Arc<Chunk>>,
    in_flight: HashSet<ChunkCoord>,
    task_tx: Sender<Task>,
    result_rx: Receiver<TaskResult>,

    // Stats
    meshing: usize,
    total_generated: usize,
    avg_gen_us: f32,
    total_meshed: usize,
    avg_mesh_us: f32,
}

impl ChunkManager {
    const NUM_WORKERS: u8 = 2;
    pub fn new(registry: Arc<BlockRegistry>) -> Self {
        let chunks = HashMap::new();
        let in_flight = HashSet::new();
        let generator = WorldGenerator::new(42);
        let (task_tx, task_rx) = mpsc::channel::<Task>();
        let (result_tx, result_rx) = mpsc::channel::<TaskResult>();

        let task_rx = Arc::new(Mutex::new(task_rx));
        for _ in 0..Self::NUM_WORKERS {
            let thread_rx = Arc::clone(&task_rx);
            let thread_result_tx = result_tx.clone();
            let thread_world_generator = generator.clone();
            let thread_registry = registry.clone();
            thread::spawn(move || {
                Self::worker_thread(thread_rx, thread_result_tx, thread_world_generator, thread_registry);
            });
        }

        Self {
            chunks,
            in_flight,
            task_tx,
            result_rx,
            meshing: 0,
            total_generated: 0,
            avg_gen_us: 0.0,
            total_meshed: 0,
            avg_mesh_us: 0.0
        }
    }

    pub fn update(&mut self, frame_event: &mut FrameEvent) {
        while let Ok(result) = self.result_rx.try_recv() {
            match result {
                TaskResult::ChunkGenerated { coord, chunk, duration } => {
                    let chunk = Arc::new(chunk);
                    self.chunks.insert(coord, chunk.clone());
                    self.in_flight.remove(&coord);

                    self.avg_gen_us = ((self.avg_gen_us * self.total_generated as f32) + duration.as_micros() as f32) / (self.total_generated as f32 + 1.0);
                    self.meshing += 1;
                    self.total_generated += 1;

                    self.task_tx.send(Task::MeshChunk { coord, chunk }).unwrap();
                },
                TaskResult::ChunkMeshed { coord, mesh, duration } => {

                    self.avg_mesh_us = ((self.avg_mesh_us * self.total_meshed as f32) + duration.as_micros() as f32) / (self.total_meshed as f32 + 1.0);
                    self.meshing -= 1;
                    self.total_meshed += 1;

                    if self.chunks.contains_key(&coord) {
                        frame_event.chunk_events.push(ChunkEvent::ChunkLoaded { coord, mesh });
                    }
                }
            }
        }
    }

    pub fn request_chunk(&mut self, coord: ChunkCoord) {
        if !self.chunks.contains_key(&coord) && !self.in_flight.contains(&coord) {
            self.in_flight.insert(coord);
            let _ = self.task_tx.send(Task::GenerateChunk { coord });
        }
    }

    pub fn retain_chunks(&mut self, desired: HashSet<ChunkCoord>, frame_event: &mut FrameEvent) {
        self.chunks.retain(|coord, _| {
            if !desired.contains(coord) {
                frame_event.chunk_events.push(ChunkEvent::ChunkUnloaded { coord: *coord });
                false
            } else {
                true
            }
        });
    }

    pub fn unload_chunk(&mut self, coord: ChunkCoord, frame_event: &mut FrameEvent) {
        if self.chunks.remove(&coord).is_some() {
            frame_event.chunk_events.push(ChunkEvent::ChunkUnloaded { coord });
        }
    }

    pub fn stats(&self) -> ChunkStats {
        ChunkStats { 
            loaded: self.chunks.len(),
            in_flight: self.in_flight.len(),
            meshing: self.meshing,
            avg_gen_us: self.avg_gen_us,
            avg_mesh_us: self.avg_mesh_us,
        }
    }

    fn worker_thread(
        task_rx: Arc<Mutex<Receiver<Task>>>,
        result_tx: Sender<TaskResult>,
        generator: WorldGenerator,
        registry: Arc<BlockRegistry>,
    ) {
        loop {
            let task = {
                let lock = task_rx.lock().expect("Mutex poisoned");
                match lock.recv() {
                    Ok(t) => t,
                    Err(_) => break,
                }
            };

            let result = match task {
                Task::GenerateChunk { coord } => {
                    let start = Instant::now();
                    let chunk = generator.generate_chunk(coord);
                    let duration = start.elapsed();
                    TaskResult::ChunkGenerated { coord, chunk, duration }
                },
                Task::MeshChunk { coord, chunk } => {
                    let start = Instant::now();
                    let mesh = ChunkMesher::create_mesh(&chunk, &registry);
                    let duration = start.elapsed();
                    TaskResult::ChunkMeshed { coord, mesh, duration }
                }
            };

            if let Err(_) = result_tx.send(result) {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub enum Task {
    GenerateChunk {
        coord: ChunkCoord
    },
    MeshChunk {
        coord: ChunkCoord,
        chunk: Arc<Chunk>
    }
}

#[derive(Debug)]
pub enum TaskResult {
    ChunkGenerated {
        coord: ChunkCoord,
        chunk: Chunk,
        duration: Duration,
    },
    ChunkMeshed {
        coord: ChunkCoord,
        mesh: CpuMesh,
        duration: Duration,
    }
}