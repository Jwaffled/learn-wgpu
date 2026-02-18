const QUAD_UVS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    // Bottom left
    vec2<f32>(0.0, 1.0),
    // Bottom right
    vec2<f32>(1.0, 1.0),
    // Top left
    vec2<f32>(0.0, 0.0),
    // Top right
    vec2<f32>(1.0, 0.0)    
);

// QuadCorner::BottomLeft = 0
// QuadCorner::BottomRight = 1
// QuadCorner::TopLeft = 2
// QuadCorner::TopRight = 3

struct VertexInput {
    @location(0) packed: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) face_id: u32,
    @location(1) uv: vec2<f32>,
    @location(2) texture_index: u32,
}

struct ChunkOffsetUniform {
    offset: vec3<f32>
}

struct CameraUniform {
    view_proj: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> chunk_offset: ChunkOffsetUniform;

fn unpack_position(packed: u32) -> vec3<f32> {
    let x = packed & 0x1Fu;
    let y = (packed >> 5u) & 0x1Fu;
    let z = (packed >> 10u) & 0x1Fu;

    return vec3<f32>(f32(x), f32(y), f32(z));
}

fn unpack_face(packed: u32) -> u32 {
    return (packed >> 15u) & 0x7u;
}

fn unpack_corner(packed: u32) -> u32 {
    return (packed >> 18u) & 0x3u;
}

fn unpack_tex(packed: u32) -> u32 {
    return (packed >> 20u) & 0xFFu;
}

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let local_pos = unpack_position(in.packed);
    let face = unpack_face(in.packed);
    let corner = unpack_corner(in.packed);
    let tex = unpack_tex(in.packed);

    let world_pos = local_pos + chunk_offset.offset;

    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.texture_index = tex;
    out.face_id = face;
    out.uv = QUAD_UVS[corner];
    
    return out;
}