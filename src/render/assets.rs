use std::{collections::HashMap, io::{BufReader, Cursor}};

use crate::render::{material::{Material, MaterialHandle}, mesh::{Mesh, MeshHandle}, model::{Model, ModelHandle, SubMesh}, state::PipelineLayouts, texture::{Texture, TextureHandle}, vertex::Vertex};

pub struct Assets {
    meshes: HashMap<MeshHandle, Mesh>,
    materials: HashMap<MaterialHandle, Material>,
    textures: HashMap<TextureHandle, Texture>,
    models: HashMap<ModelHandle, Model>,

    next_mesh_handle: MeshHandle,
    next_material_handle: MaterialHandle,
    next_texture_handle: TextureHandle,
    next_model_handle: ModelHandle,
}

impl Assets {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, layouts: &PipelineLayouts) -> Self {
        let meshes = HashMap::from([
            (MeshHandle(0), Mesh::cube(device))
        ]);

        let default_texture = Texture::from_rgba(device, queue, [255, 255, 255, 255], "Default texture").unwrap();

        let textures = HashMap::from([
            (TextureHandle::DEFAULT_TEXTURE, default_texture)
        ]);

        let materials = HashMap::from([
            (MaterialHandle::DEFAULT_MATERIAL, Material::new(&textures.get(&TextureHandle::DEFAULT_TEXTURE).unwrap(), device, &layouts.material))
        ]);

        

        let models = HashMap::from([]);

        Self {
            meshes,
            materials,
            textures,
            models,

            next_mesh_handle: MeshHandle(1),
            next_material_handle: MaterialHandle(1),
            next_texture_handle: TextureHandle(1),
            next_model_handle: ModelHandle(0),
        }
    }

    pub fn get_material(&self, handle: MaterialHandle) -> &Material {
        self.materials.get(&handle).unwrap()
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> &Mesh {
        self.meshes.get(&handle).unwrap()
    }

    pub fn get_texture(&self, handle: TextureHandle) -> &Texture {
        self.textures.get(&handle).unwrap()
    }

    pub fn get_model(&self, handle: ModelHandle) -> &Model {
        self.models.get(&handle).unwrap()
    }

    pub async fn load_texture(&mut self, file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<TextureHandle> {
        let bytes = Self::load_binary(file_name).await?;
        let texture = Texture::from_bytes(device, queue, &bytes, file_name)?;
        let handle = self.next_texture_handle;
        self.next_texture_handle.0 += 1;

        self.textures.insert(handle, texture);
        Ok(handle)
    }

    pub async fn load_material(&mut self, texture: TextureHandle, device: &wgpu::Device, material_layout: &wgpu::BindGroupLayout) -> anyhow::Result<MaterialHandle> {
        let material = Material::new(&self.textures[&texture], device, &material_layout);
        let handle = self.next_material_handle;
        self.next_material_handle.0 += 1;
        self.materials.insert(handle, material);
        Ok(handle)
    }

    pub async fn load_model(&mut self, file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue, material_layout: &wgpu::BindGroupLayout) -> anyhow::Result<ModelHandle> {
        let obj_text = Self::load_string(file_name).await?;
        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, obj_materials) = tobj::load_obj_buf_async(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = Self::load_string(&p).await.unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            }
        )
        .await?;

        let mut materials = Vec::new();
        for m in obj_materials? {
            let diffuse_texture = self.load_texture(&m.diffuse_texture, device, queue).await?;
            let material_handle = self.load_material(diffuse_texture, device, material_layout).await?;
            materials.push(material_handle);
        }

        
        let mut parts = Vec::new();

        for m in models {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        Vertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2]
                            ],
                            tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                            normal: [0.0, 0.0, 0.0]
                        }
                    } else {
                        Vertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2]
                            ],
                            tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2]
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();
            let mesh = Mesh::from_vertices_indices(&vertices, &m.mesh.indices, device);
            let handle = self.next_mesh_handle;
            self.next_mesh_handle.0 += 1;
            self.meshes.insert(handle, mesh);
            let material_handle = match m.mesh.material_id {
                Some(id) if id < materials.len() => materials[id],
                _ => MaterialHandle::DEFAULT_MATERIAL
            };

            parts.push(SubMesh {
                mesh: handle,
                material: material_handle
            })
        }

        let model_handle = self.next_model_handle;
        self.next_model_handle.0 += 1;
        self.models.insert(model_handle, Model { parts });

        Ok(model_handle)
    }

    async fn load_string(file_name: &str) -> anyhow::Result<String> {
        let txt = {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);

            std::fs::read_to_string(path)?
        };

        Ok(txt)
    }

    async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
        let data = {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);

            std::fs::read(path)?
        };

        Ok(data)
    }
}