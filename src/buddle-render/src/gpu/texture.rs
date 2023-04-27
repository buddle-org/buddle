use bcndecode::{BcnDecoderFormat, BcnEncoding};
use buddle_math::UVec2;
use buddle_nif::enums::PixelFormat;
use buddle_nif::Nif;
use buddle_nif::objects::NiObject;
use crate::{Context, TextureDimensions};

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) dimensions: TextureDimensions,
    pub(crate) size: UVec2,
}

impl Texture {
    pub fn from_ni_texturing_property(
        ctx: &Context,
        property: &NiObject,
        nif: &Nif,
    ) -> Result<(Self, bool, bool), ()> {
        let NiObject::NiTexturingProperty(texturing) = property else {
            return Err(());
        };

        let base_texture = texturing.base_texture.as_ref().ok_or(())?;

        let NiObject::NiSourceTexture(source) = base_texture.source.get_or(&nif.blocks, ())? else {
            return Err(());
        };

        let NiObject::NiPixelData(pixel_data) = source.pixel_data.get_or(&nif.blocks, ())? else {
            return Err(());
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
