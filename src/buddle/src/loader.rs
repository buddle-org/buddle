//! Convert NIF objects to buddle-render ones

use std::io;
use buddle_math::{UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};
use buddle_nif::enums::{AlphaFunction, PixelFormat};
use buddle_nif::objects::{NiAlphaProperty, NiObject, NiPixelData};
use buddle_nif::Nif;
use buddle_render::{Context, FlatMaterial, Material, Model, Texture, Transform, Vertex};

use bcndecode::{BcnDecoderFormat, BcnEncoding};
use buddle_wad::{Archive, Interner};

pub trait ToModel {
    type Error;

    fn to_model(self, ctx: &Context) -> Result<Model, Self::Error>;
}

pub trait ToTexture {
    type Error;

    fn to_texture(self, ctx: &Context) -> Result<(Texture, bool, bool), Self::Error>;
}

// Todo: Speedups
fn get_child_meshes_with_transforms<'a>(
    nif: &'a Nif,
    object: &'a NiObject,
    mut transform: Transform,
) -> Vec<(&'a NiObject, Transform)> {
    let Some(av) = object.avobject() else { return Vec::new() };

    transform = transform * Transform::from_nif(av);

    let Some(children) = object.child_refs() else { return Vec::new() };

    let mut res = Vec::new();

    for child in children {
        if let Some(child_obj) = child.get(&nif.blocks) {
            if let NiObject::NiMesh(mesh) = child_obj {
                res.push((child_obj, transform * Transform::from_nif(&mesh.base.base)));
            } else {
                res.append(&mut get_child_meshes_with_transforms(
                    nif,
                    child.get(&nif.blocks).unwrap(),
                    transform,
                ))
            }
        }
    }

    res
}

fn get_meshes_with_transforms(nif: &Nif) -> Vec<(&NiObject, Transform)> {
    let mut res = Vec::new();
    for root in &nif.root_objects() {
        res.append(&mut get_child_meshes_with_transforms(
            nif,
            root,
            Transform::default(),
        ))
    }
    res
}

fn blend_state_from_alpha_property(alpha: &NiAlphaProperty) -> Option<wgpu::BlendState> {
    if !alpha.flags.alpha_blend() {
        return None;
    }

    let src = alpha.flags.source_blend_mode();
    let dst = alpha.flags.destination_blend_mode();

    let mut res = wgpu::BlendState::REPLACE;

    res.color.src_factor = match src {
        AlphaFunction::ALPHA_ONE => wgpu::BlendFactor::One,
        AlphaFunction::ALPHA_ZERO => wgpu::BlendFactor::Zero,
        AlphaFunction::ALPHA_SRC_COLOR => wgpu::BlendFactor::Src,
        AlphaFunction::ALPHA_INV_SRC_COLOR => wgpu::BlendFactor::OneMinusSrc,
        AlphaFunction::ALPHA_DEST_COLOR => wgpu::BlendFactor::Dst,
        AlphaFunction::ALPHA_INV_DEST_COLOR => wgpu::BlendFactor::OneMinusDst,
        AlphaFunction::ALPHA_SRC_ALPHA => wgpu::BlendFactor::SrcAlpha,
        AlphaFunction::ALPHA_INV_SRC_ALPHA => wgpu::BlendFactor::OneMinusSrcAlpha,
        AlphaFunction::ALPHA_DEST_ALPHA => wgpu::BlendFactor::DstAlpha,
        AlphaFunction::ALPHA_INV_DEST_ALPHA => wgpu::BlendFactor::OneMinusDstAlpha,
        AlphaFunction::ALPHA_SRC_ALPHA_SATURATE => wgpu::BlendFactor::SrcAlphaSaturated,
    };

    res.color.dst_factor = match dst {
        AlphaFunction::ALPHA_ONE => wgpu::BlendFactor::One,
        AlphaFunction::ALPHA_ZERO => wgpu::BlendFactor::Zero,
        AlphaFunction::ALPHA_SRC_COLOR => wgpu::BlendFactor::Src,
        AlphaFunction::ALPHA_INV_SRC_COLOR => wgpu::BlendFactor::OneMinusSrc,
        AlphaFunction::ALPHA_DEST_COLOR => wgpu::BlendFactor::Dst,
        AlphaFunction::ALPHA_INV_DEST_COLOR => wgpu::BlendFactor::OneMinusDst,
        AlphaFunction::ALPHA_SRC_ALPHA => wgpu::BlendFactor::SrcAlpha,
        AlphaFunction::ALPHA_INV_SRC_ALPHA => wgpu::BlendFactor::OneMinusSrcAlpha,
        AlphaFunction::ALPHA_DEST_ALPHA => wgpu::BlendFactor::DstAlpha,
        AlphaFunction::ALPHA_INV_DEST_ALPHA => wgpu::BlendFactor::OneMinusDstAlpha,
        AlphaFunction::ALPHA_SRC_ALPHA_SATURATE => wgpu::BlendFactor::SrcAlphaSaturated,
    };

    Some(res)
}

impl ToModel for (Nif, &mut Interner<&Archive>) {
    type Error = ();

    fn to_model(self, ctx: &Context) -> Result<Model, Self::Error> {
        let (nif, intern) = self;

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
            let mut color_regions = Vec::new();

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
                } else if kind.data.starts_with("COLOR") {
                    color_regions = datastream.read_color4();
                }
            }

            let mut indices = Vec::new();
            let mut vertices = Vec::new();

            let mut base_index = 0u16;
            let region_count = index_regions.len();

            // This naively assumes that in any Nif there are either tex coords for all vertices or none
            // If a vertex region other than the last misses tex coords, this assumption breaks and
            // tex coords are used and generated for the wrong vertices
            if vertex_regions.len() > tex_coords_regions.len() {
                let start = tex_coords_regions.len();

                for vertex_region in vertex_regions.iter().skip(start) {
                    let mut tex_coords = Vec::new();

                    for vertex in vertex_region {
                        tex_coords.push(Vec2::new(vertex.x, vertex.y) * 0.01);
                    }

                    tex_coords_regions.push(tex_coords);
                }
            }

            if vertex_regions.len() > normal_regions.len() {
                let start = normal_regions.len();

                for vertex_region in vertex_regions.iter().skip(start) {
                    let mut normals = Vec::new();

                    for _ in vertex_region {
                        // How much harm could that possibly do, we're not even shading yet
                        // Todo: actually calculate the normals
                        normals.push(Vec3::new(0.0, 0.0, 0.0));
                    }

                    normal_regions.push(normals);
                }
            }

            if vertex_regions.len() > color_regions.len() {
                let start = color_regions.len();

                for vertex_region in vertex_regions.iter().skip(start) {
                    let mut colors = Vec::new();

                    for _ in vertex_region {
                        colors.push(Vec4::new(1.0, 1.0, 1.0, 1.0));
                    }

                    color_regions.push(colors);
                }
            }

            for i in 0..region_count {
                // Indices only reference vertices in their own region, so we have to offset them
                for index in index_regions.get(i).ok_or(())? {
                    indices.push(base_index + index);
                }

                let vertex_region = vertex_regions.get(i).ok_or(())?;
                let count = vertex_region.len();

                for j in 0..count {
                    let mut in_file_pos = *vertex_region.get(j).ok_or(())?;
                    in_file_pos = transform.rotation.mul_vec3(in_file_pos);

                    // W101's up is Z so swap that
                    let mut pos = in_file_pos * transform.scale + transform.translation;
                    std::mem::swap(&mut pos.z, &mut pos.y);
                    std::mem::swap(&mut pos.x, &mut pos.z);
                    pos *= 0.01;

                    let mut normal = normal_regions.get(i).ok_or(())?.get(j).ok_or(())?.clone();
                    std::mem::swap(&mut normal.z, &mut normal.y);
                    std::mem::swap(&mut normal.x, &mut normal.z);

                    vertices.push(Vertex::new(
                        pos,
                        color_regions.get(i).ok_or(())?.get(j).ok_or(())?.xyz(),
                        *normal_regions.get(i).ok_or(())?.get(j).ok_or(())?,
                        *tex_coords_regions.get(i).ok_or(())?.get(j).ok_or(())?,
                    ))
                }
                base_index += count as u16;
            }

            let mut texture = Err(());
            let mut alpha = None;

            for property in properties {
                if let NiObject::NiAlphaProperty(alpha_prop) = property {
                    alpha = Some(alpha_prop);
                }

                if texture.is_err() {
                    let texturing = match property {
                        NiObject::NiTexturingProperty(prop) => prop,
                        NiObject::NiMultiTextureProperty(multi_prop) => &multi_prop.base,
                        _ => continue,
                    };

                    let base_texture = texturing.base_texture.as_ref().ok_or(())?;

                    let NiObject::NiSourceTexture(source) = base_texture.source.get_or(&nif.blocks, ())? else {
                        continue;
                    };

                    let pixel_data = if source.use_external == 1 {
                        let file_name = "Textures/".to_string()
                            + &nif.header.strings[source.file_name.index.0 as usize]
                            .data
                            .clone();
                        let handle = intern.intern(&file_name).map_err(|_| ())?;
                        let data = intern.fetch_mut(handle).unwrap();
                        let mut cursor = io::Cursor::new(data);
                        let nif = Nif::parse(&mut cursor).map_err(|_| ())?;

                        match &nif.root_objects()[0] {
                            NiObject::NiPixelData(pd) => pd.clone(),
                            _ => continue,
                        }
                    } else {
                        match source.pixel_data.get_or(&nif.blocks, ())? {
                            NiObject::NiPixelData(pd) => pd.clone(),
                            _ => continue,
                        }
                    };

                    texture = pixel_data.to_texture(ctx);
                }
            }

            let blend = if let Some(alpha) = alpha {
                blend_state_from_alpha_property(alpha)
            } else {
                None
            };

            // fixme: there exist models without textures that are duplicates of and at the same
            //  position as other models. why?
            let texture = texture.unwrap_or_else(|_| (Texture::missing(ctx), false, true));
            let material: Box<dyn Material> = Box::new(FlatMaterial::new(ctx, &texture.0, blend, texture.1, texture.2));

            let mesh = ctx.create_mesh(vertices, indices);

            meshes.push(mesh);
            materials.push(material);
            textures.push(texture.0);
        }

        Ok(Model::new(meshes, materials, textures))
    }
}

impl ToTexture for NiPixelData {
    type Error = ();

    fn to_texture(self, ctx: &Context) -> Result<(Texture, bool, bool), Self::Error> {
        let mm = self.mipmaps.get(0).ok_or(())?;
        let size = UVec2::new(mm.width, mm.height);

        let pixels;

        if self.base.pixel_format == PixelFormat::PX_FMT_DXT1
            || self.base.pixel_format == PixelFormat::PX_FMT_DXT3
            || self.base.pixel_format == PixelFormat::PX_FMT_DXT5
        {
            pixels = bcndecode::decode(
                &self.pixel_data,
                mm.width as usize,
                mm.height as usize,
                match self.base.pixel_format {
                    PixelFormat::PX_FMT_DXT1 => BcnEncoding::Bc1,
                    PixelFormat::PX_FMT_DXT3 => BcnEncoding::Bc2,
                    PixelFormat::PX_FMT_DXT5 => BcnEncoding::Bc3,
                    _ => unreachable!(),
                },
                BcnDecoderFormat::RGBA,
            )
            .map_err(|_| ())?;
        } else {
            return Err(());
        }

        let mut iter = pixels.iter();
        iter.advance_by(3).expect("TODO: panic message");

        let mut transparent = false;
        let mut opaque = false;

        for d in iter.step_by(4) {
            if *d < 255u8 {
                transparent = true;
            } else {
                opaque = true;
            }

            if transparent == true && opaque == true {
                break;
            }
        }

        Ok((ctx.create_texture(&pixels, size), transparent, opaque))
    }
}
