use crate::gles2::conversions::GL_CUBE_MAP_TARGETS;
use crate::gles2::gles2_bindings::types::GLenum;
use crate::gles2::{gles2_bindings, RafxDeviceContextGles2, TextureId, NONE_TEXTURE};
use crate::{
    GlTextureFormatInfo, RafxResourceType, RafxResult, RafxSampleCount, RafxTextureDef,
    RafxTextureDimensions,
};
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum RafxRawImageGles2 {
    //Renderbuffer(RenderbufferId),
    Texture(TextureId),
}

impl RafxRawImageGles2 {
    pub fn gl_texture_id(&self) -> Option<TextureId> {
        match self {
            //RafxRawImageGl::Renderbuffer(_) => None,
            RafxRawImageGles2::Texture(id) => Some(*id),
        }
    }

    // pub fn gl_renderbuffer_id(&self) -> Option<RenderbufferId> {
    //     match self {
    //         //RafxRawImageGl::Renderbuffer(id) => Some(*id),
    //         RafxRawImageGl::Texture(_) => None,
    //     }
    // }
}

#[derive(Debug)]
pub struct RafxTextureGles2Inner {
    device_context: RafxDeviceContextGles2,
    texture_def: RafxTextureDef,
    image: RafxRawImageGles2,
    gl_target: GLenum,
    texture_id: u32,
    format_info: GlTextureFormatInfo,
}

impl Drop for RafxTextureGles2Inner {
    fn drop(&mut self) {
        match self.image {
            //RafxRawImageGl::Renderbuffer(_) => {} // do nothing
            RafxRawImageGles2::Texture(texture_id) => self
                .device_context
                .gl_context()
                .gl_destroy_texture(texture_id)
                .unwrap(),
        }
    }
}

/// Holds the vk::Image and allocation as well as a few vk::ImageViews depending on the
/// provided RafxResourceType in the texture_def.
#[derive(Clone, Debug)]
pub struct RafxTextureGles2 {
    inner: Arc<RafxTextureGles2Inner>,
}

impl PartialEq for RafxTextureGles2 {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.inner.texture_id == other.inner.texture_id
    }
}

impl Eq for RafxTextureGles2 {}

impl Hash for RafxTextureGles2 {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.inner.texture_id.hash(state);
    }
}

impl RafxTextureGles2 {
    pub fn texture_def(&self) -> &RafxTextureDef {
        &self.inner.texture_def
    }

    pub fn gl_raw_image(&self) -> &RafxRawImageGles2 {
        &self.inner.image
    }

    pub fn gl_target(&self) -> GLenum {
        self.inner.gl_target
    }

    pub fn gl_format_info(&self) -> &GlTextureFormatInfo {
        &self.inner.format_info
    }

    pub fn new(
        device_context: &RafxDeviceContextGles2,
        texture_def: &RafxTextureDef,
    ) -> RafxResult<RafxTextureGles2> {
        Self::from_existing(device_context, None, texture_def)
    }

    // This path is mostly so we can wrap a provided swapchain image
    pub fn from_existing(
        device_context: &RafxDeviceContextGles2,
        existing_image: Option<RafxRawImageGles2>,
        texture_def: &RafxTextureDef,
    ) -> RafxResult<RafxTextureGles2> {
        texture_def.verify();

        if texture_def.sample_count != RafxSampleCount::SampleCount1 {
            unimplemented!("GL ES 2.0 backend does not implement multisampled images");
        }

        let dimensions = texture_def
            .dimensions
            .determine_dimensions(texture_def.extents);

        if dimensions != RafxTextureDimensions::Dim2D {
            unimplemented!("GL ES 2.0 only supports 2D textures");
        }

        let gl_target = if texture_def
            .resource_type
            .contains(RafxResourceType::TEXTURE_CUBE)
        {
            if texture_def.array_length != 6 {
                unimplemented!("GL ES 2.0 does not support cube map arrays");
            }
            gles2_bindings::TEXTURE_CUBE_MAP
        } else {
            gles2_bindings::TEXTURE_2D
        };

        let format_info = texture_def
            .format
            .gles2_texture_format_info()
            .ok_or_else(|| format!("Format {:?} not supported", texture_def.format))?;

        let image = if let Some(existing_image) = existing_image {
            existing_image
        } else {
            //TODO: glTexStorage2D/3D (ES3 only)
            //multisample support
            //TODO: Mipmaps
            let gl_context = device_context.gl_context();
            let texture_id = gl_context.gl_create_texture()?;
            gl_context.gl_pixel_storei(gles2_bindings::UNPACK_ALIGNMENT, 1)?;

            // If it's a cubemap, the gl_tex_image_2d() call takes a different target enum than the
            // gl_bind_texture() call
            let subtargets = if gl_target == gles2_bindings::TEXTURE_CUBE_MAP {
                &GL_CUBE_MAP_TARGETS[..]
            } else {
                &[gles2_bindings::TEXTURE_2D]
            };

            gl_context.gl_bind_texture(gl_target, texture_id)?;
            for &subtarget in subtargets {
                //TODO: Compressed texture support?

                for mip_level in 0..texture_def.mip_count {
                    gl_context.gl_tex_image_2d(
                        subtarget,
                        mip_level as u8,
                        format_info.gl_internal_format,
                        texture_def.extents.width >> mip_level,
                        texture_def.extents.height >> mip_level,
                        0,
                        format_info.gl_format,
                        format_info.gl_type,
                        None,
                    )?;
                }
            }
            gl_context.gl_bind_texture(gl_target, NONE_TEXTURE)?;

            RafxRawImageGles2::Texture(texture_id)
        };

        let texture_id = crate::internal_shared::NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed);

        let inner = RafxTextureGles2Inner {
            device_context: device_context.clone(),
            image,
            texture_def: texture_def.clone(),
            gl_target,
            texture_id,
            format_info,
        };

        return Ok(RafxTextureGles2 {
            inner: Arc::new(inner),
        });
    }
}
