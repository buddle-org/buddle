//! Combines materials and meshes to easily render objects

use buddle_math::{Mat4};

use crate::{Material, Mesh, RenderBuffer, Texture};

pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Box<dyn Material>>,
    _owned_textures: Vec<Texture>,
}

impl Model {
    pub fn new(
        meshes: Vec<Mesh>,
        materials: Vec<Box<dyn Material>>,
        _owned_textures: Vec<Texture>,
    ) -> Self {
        Model {
            meshes,
            materials,
            _owned_textures,
        }
    }

    pub fn from_mesh(mesh: Mesh, material: Box<dyn Material>) -> Self {
        Model {
            meshes: vec![mesh],
            materials: vec![material],
            _owned_textures: vec![],
        }
    }

    pub fn render_to<'a, 'b>(
        &'a self,
        render_buffer: &'b mut RenderBuffer<'a, 'a>,
        model_matrix: Mat4,
    ) {
        for i in 0..self.meshes.len() {
            render_buffer.add_draw_call(&self.meshes[i], &self.materials[i], model_matrix);
        }
    }
}
