use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex, mpsc::{self, Receiver, Sender}}, thread, time::{Duration, Instant}};

use crate::{game::{chunk::{Block, Chunk, ChunkCoord, LocalCoord}, generator::WorldGenerator, player::Player, world::{ChunkEvent, FrameEvent, WorldState}}, render::{mesh::CpuMesh, renderers::debug_text::ChunkStats, vertex::Vertex}};

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
    pub fn create_mesh(chunk: &Chunk, chunk_coord: ChunkCoord) -> CpuMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let (chunk_x, chunk_y, chunk_z) = chunk_coord.as_tuple();
        let (chunk_offset_x, chunk_offset_y, chunk_offset_z) = (chunk_x * Chunk::CHUNK_SIZE as isize, chunk_y * Chunk::CHUNK_HEIGHT as isize, chunk_z * Chunk::CHUNK_SIZE as isize);

        for y in 0..Chunk::CHUNK_HEIGHT as isize {
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

    pub fn create_player_mesh(player: &Player) -> CpuMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let size = 0.6;

        let px = player.position.x;
        let py = player.position.y;
        let pz = player.position.z;

        let (uv_min, uv_max) = Block::Dirt.get_tile().uv_rect();
        let uvs = [
            [uv_min[0], uv_min[1]],
            [uv_max[0], uv_min[1]],
            [uv_max[0], uv_max[1]],
            [uv_min[0], uv_max[1]],
        ];

        for face in &FACES {
            let base_index = vertices.len() as u32;

            for (i, corner) in face.corners.iter().enumerate() {
                let x = (corner[0] - 0.5) * size + px;
                let y = (corner[1] - 0.5) * size + py;
                let z = (corner[2] - 0.5) * size + pz;

                vertices.push(Vertex {
                    position: [x, y, z],
                    tex_coords: uvs[i],
                    normal: face.normal,
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

        CpuMesh { vertices, indices }
    }
}

pub struct ChunkManager {
    chunks: HashMap<ChunkCoord, Chunk>,
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
    pub fn new() -> Self {
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
            thread::spawn(move || {
                Self::worker_thread(thread_rx, thread_result_tx, thread_world_generator);
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
        generator: WorldGenerator
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
                    let mesh = ChunkMesher::create_mesh(&chunk, coord);
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
        chunk: Chunk
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