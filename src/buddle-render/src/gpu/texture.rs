use crate::{Context, TextureDimensions};
use bcndecode::{BcnDecoderFormat, BcnEncoding};
use buddle_math::UVec2;
use buddle_nif::enums::PixelFormat;
use buddle_nif::objects::NiObject;
use buddle_nif::Nif;
use buddle_wad::{Archive, Interner};
use std::io;

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) dimensions: TextureDimensions,
    pub(crate) size: UVec2,
}

impl Texture {
    pub fn missing(ctx: &Context) -> Self {
        ctx.create_texture(
            &[
                0, 0, 0, 255, 255, 0, 220, 255, 255, 0, 220, 255, 0, 0, 0, 255,
            ],
            UVec2::new(2, 2),
        )
    }

    pub fn from_ni_texturing_property(
        ctx: &Context,
        property: &NiObject,
        intern: &mut Interner<&Archive>,
        nif: &Nif,
    ) -> Result<(Self, bool, bool), ()> {
        let texturing = match property {
            NiObject::NiTexturingProperty(prop) => prop,
            NiObject::NiMultiTextureProperty(multi_prop) => &multi_prop.base,
            _ => return Err(()),
        };

        let base_texture = texturing.base_texture.as_ref().ok_or(())?;

        let NiObject::NiSourceTexture(source) = base_texture.source.get_or(&nif.blocks, ())? else {
            return Err(());
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
                _ => return Err(()),
            }
        } else {
            match source.pixel_data.get_or(&nif.blocks, ())? {
                NiObject::NiPixelData(pd) => pd.clone(),
                _ => return Err(()),
            }
        };

        let mm = pixel_data.mipmaps.get(0).ok_or(())?;
        let size = UVec2::new(mm.width, mm.height);

        let mut pixels = pixel_data.pixel_data.clone();

        if pixel_data.base.pixel_format == PixelFormat::PX_FMT_DXT1
            || pixel_data.base.pixel_format == PixelFormat::PX_FMT_DXT3
            || pixel_data.base.pixel_format == PixelFormat::PX_FMT_DXT5
        {
            pixels = bcndecode::decode(
                &pixel_data.pixel_data,
                mm.width as usize,
                mm.height as usize,
                match pixel_data.base.pixel_format {
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
