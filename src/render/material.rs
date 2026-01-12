use crate::render::{assets::Assets, texture::{Texture, TextureHandle}};

pub struct Material {
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(diffuse_texture: &Texture, device: &wgpu::Device, textures_bind_group_layout: &wgpu::BindGroupLayout) -> Self {        
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("material_bind_group"),
                layout: textures_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view)
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler)
                    }
                ]
            }
        );


        Self {
            bind_group,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MaterialHandle(pub u32);

impl MaterialHandle {
    pub const DEFAULT_MATERIAL: MaterialHandle = MaterialHandle(0);
}