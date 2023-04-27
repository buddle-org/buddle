//! Combines materials and meshes to easily render objects


use buddle_math::{Mat4, Quat, Vec3};
use buddle_nif::basic::Ref;
use buddle_nif::compounds::Vector3;
use buddle_nif::objects::{NiAVObject, NiObject};
use buddle_nif::Nif;

use crate::{Context, FlatMaterial, Material, Mesh, RenderBuffer, Texture, Transform, Vertex};

pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Box<dyn Material>>,
    _owned_textures: Vec<Texture>,
}

// Todo: Speedups
fn get_child_meshes_with_transforms(
    nif: &Nif,
    obj: Ref<NiAVObject>,
    mut transform: Transform,
) -> Vec<(&NiObject, Transform)> {
    let Some(object) = obj.get(&nif.blocks) else { return Vec::new() };

    let Some(av) = object.avobject() else { return Vec::new() };

    transform.translation += <Vector3 as Into<Vec3>>::into(av.translation.clone());
    transform.rotation *= Quat::from_mat3(&av.rotation.clone().into());
    transform.scale *= av.scale;

    let Some(children) = object.child_refs() else { return Vec::new() };

    let mut res = Vec::new();

    for child in children {
        if let Some(child_obj) = child.get(&nif.blocks) {
            if let NiObject::NiMesh(mesh) = child_obj {
                let mut mesh_transform = transform;
                mesh_transform.translation +=
                    <Vector3 as Into<Vec3>>::into(mesh.base.base.translation.clone());
                mesh_transform.rotation *= Quat::from_mat3(&mesh.base.base.rotation.clone().into());
                mesh_transform.scale *= mesh.base.base.scale;
                res.push((child_obj, mesh_transform));
            } else {
                res.append(&mut get_child_meshes_with_transforms(
                    nif,
                    child.clone(),
                    transform,
                ))
            }
        }
    }

    res
}

fn get_meshes_with_transforms(nif: &Nif) -> Vec<(&NiObject, Transform)> {
    let mut res = Vec::new();
    for root in &nif.footer.roots {
        let Some(object) = root.get(&nif.blocks) else { return Vec::new() };

        let Some(av) = object.avobject() else { return Vec::new() };

        let transform = Transform::from_nif(
            av.translation.clone(),
            Quat::from_mat3(&av.rotation.clone().into()),
            av.scale,
        );

        let children = object.child_refs().unwrap();

        for child in children {
            if let Some(child_obj) = child.get(&nif.blocks) {
                if let NiObject::NiMesh(_) = child_obj {
                    res.push((child_obj, transform));
                } else {
                    res.append(&mut get_child_meshes_with_transforms(
                        nif,
                        child.clone(),
                        transform,
                    ))
                }
            }
        }
    }
    res
}

impl Model {
    pub fn from_mesh(mesh: Mesh, material: Box<dyn Material>) -> Self {
        Model {
            meshes: vec![mesh],
            materials: vec![material],
            _owned_textures: vec![],
        }
    }

    pub fn from_nif(ctx: &Context, nif: Nif) -> Result<Self, ()> {
        let mut meshes = Vec::new();
        let mut materials = Vec::new();
        let mut textures = Vec::new();

        let ni_meshes = get_meshes_with_transforms(&nif);

        for (ni_mesh, transform) in ni_meshes {
            let properties = nif.properties_for(&ni_mesh).unwrap();

            let ni_mesh = match ni_mesh {
                NiObject::NiMesh(mesh) => mesh,
                _ => unreachable!(),
            };

            let mut index_regions = Vec::new();
            let mut vertex_regions = Vec::new();
            let mut tex_coords_regions = Vec::new();
            let mut normal_regions = Vec::new();

            for ds_ref in &ni_mesh.datastreams {
                let datastream = {
                    match nif.blocks.get(ds_ref.stream.0 as usize).ok_or(())? {
                        NiObject::NiDataStream(datastream) => datastream,
                        _ => return Err(()),
                    }
                };

                let semantic_data = ds_ref.component_semantics.get(0).ok_or(())?;
                let kind = nif
                    .header
                    .strings
                    .get(semantic_data.name.0 as usize)
                    .ok_or(())?;

                if kind.data == "INDEX" {
                    index_regions = datastream.read_primitive::<u16>();
                } else if kind.data == "TEXCOORD" {
                    tex_coords_regions = datastream.read_vec2();
                } else if kind.data.starts_with("POSITION") {
                    vertex_regions = datastream.read_vec3();
                } else if kind.data.starts_with("NORMAL") {
                    normal_regions = datastream.read_vec3();
                }
            }

            let mut indices = Vec::new();
            let mut vertices = Vec::new();

            let mut base_index = 0u16;
            let region_count = index_regions.len();

            for i in 0..region_count {
                // Indices only reference vertices in their own region, so we have to offset them
                for index in index_regions.get(i).ok_or(())? {
                    indices.push(base_index + index);
                }

                let count = vertex_regions.get(i).ok_or(())?.len();
                for j in 0..count {
                    // W101's up is Z so swap that
                    let mut in_file_pos = *vertex_regions.get(i).ok_or(())?.get(j).ok_or(())?;
                    in_file_pos = transform.rotation.mul_vec3(in_file_pos);

                    let mut pos = in_file_pos + transform.translation;
                    std::mem::swap(&mut pos.z, &mut pos.y);

                    let mut normal = normal_regions.get(i).ok_or(())?.get(j).ok_or(())?.clone();
                    std::mem::swap(&mut normal.z, &mut normal.y);
                    normal.z *= -1.0;
                    normal.x *= -1.0;
                    vertices.push(Vertex::new(
                        pos,
                        Vec3::ZERO,
                        normal,
                        *tex_coords_regions.get(i).ok_or(())?.get(j).ok_or(())?,
                    ))
                }
                base_index += count as u16;
            }

            let mut texture = Err(());

            for property in properties {
                texture = Texture::from_ni_texturing_property(ctx, property, &nif);
                if texture.is_ok() {
                    break;
                }
            }
            let texture = texture?;
            let material: Box<dyn Material> = Box::new(FlatMaterial::new(ctx, &texture.0, texture.1, texture.2));

            let mesh = ctx.create_mesh(&vertices, &indices);

            meshes.push(mesh);
            materials.push(material);
            textures.push(texture.0);
        }

        Ok(Model {
            meshes,
            materials,
            _owned_textures: textures,
        })
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
