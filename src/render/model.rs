use std::ops::Range;

use crate::render::{assets::Assets, material::{Material, MaterialHandle}, mesh::{Mesh, MeshHandle}, state::PipelineLayouts};

pub struct Model {
    pub parts: Vec<SubMesh>
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ModelHandle(pub u32);

pub struct SubMesh {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, material: &'a Material, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup
    );

    fn draw_model(&mut self, model: ModelHandle, assets: &'a Assets, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_model_instanced(&mut self, model: ModelHandle, assets: &'a Assets, instances: Range<u32>, camera_bind_group: &'a wgpu::BindGroup);
}

impl <'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where 'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, camera_bind_group: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group);
    }

    fn draw_mesh_instanced(
            &mut self,
            mesh: &'b Mesh,
            material: &'b Material,
            instances: Range<u32>,
            camera_bind_group: &'b wgpu::BindGroup
        ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(PipelineLayouts::CAMERA_SLOT, camera_bind_group, &[]);
        self.set_bind_group(PipelineLayouts::MATERIAL_SLOT, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.index_count, 0, instances);
    }

    fn draw_model(&mut self, model: ModelHandle, assets: &'b Assets, camera_bind_group: &'b wgpu::BindGroup) {
        self.draw_model_instanced(model, assets, 0..1, camera_bind_group);
    }

    fn draw_model_instanced(&mut self, model: ModelHandle, assets: &'b Assets, instances: Range<u32>, camera_bind_group: &'b wgpu::BindGroup) {
        let model = assets.get_model(model);
        for submesh in &model.parts {
            let material = assets.get_material(submesh.material);
            let mesh = assets.get_mesh(submesh.mesh);
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group);
        }
    }
}