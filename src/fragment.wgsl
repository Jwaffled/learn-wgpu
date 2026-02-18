@group(1) @binding(0)
var block_textures: texture_2d_array<f32>;
@group(1) @binding(1)
var block_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let color = textureSample(
        block_textures,
        block_sampler,
        in.uv,
        i32(in.texture_index),
    );

    return color;
}