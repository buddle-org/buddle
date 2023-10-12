//! Interfaces with the GPU

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::util::DeviceExt;

use buddle_math::{Mat4, UVec2};

use crate::camera::ModelMatrices;
use crate::gpu::*;

pub struct Context {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: Surface,
    pub(crate) depth_buffer: Texture,
    pub(crate) oit_opaque: Texture,
    pub(crate) oit_accum: Texture,
    pub(crate) oit_reveal: Texture,
    shader_cache: RefCell<HashMap<(&'static str, SimplifiedPipelineConfig), Rc<Shader>>>,
}

impl Context {
    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(window: &W, size: UVec2) -> Self {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            format: surface_format,
            width: size.x,
            height: size.y,
            // todo: control vsync properly
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_buffer = Self::create_surface_depth_texture(&device, &config);
        let oit_opaque = Self::create_oit_opaque_texture(&device, &config);
        let oit_accum = Self::create_oit_accum_texture(&device, size);
        let oit_reveal = Self::create_oit_reveal_texture(&device, size);

        Context {
            device,
            queue,
            surface: Surface { surface, config },
            depth_buffer,
            oit_opaque,
            oit_accum,
            oit_reveal,
            shader_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Resizes the internal surface
    pub fn resize(&mut self, new_size: UVec2) {
        if new_size.x > 0 && new_size.y > 0 {
            self.surface.config.width = new_size.x;
            self.surface.config.height = new_size.y;

            self.depth_buffer =
                Self::create_surface_depth_texture(&self.device, &self.surface.config);
            self.oit_opaque = Self::create_oit_opaque_texture(&self.device, &self.surface.config);
            self.oit_accum = Self::create_oit_accum_texture(&self.device, new_size);
            self.oit_reveal = Self::create_oit_reveal_texture(&self.device, new_size);

            self.reconfigure();
        }
    }

    pub fn reconfigure(&self) {
        self.surface.configure(&self.device);
    }

    pub fn create_buffer<T>(&self, data: &[T], usage: wgpu::BufferUsages) -> wgpu::Buffer
    where
        T: bytemuck::Pod,
        T: bytemuck::Zeroable,
    {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Buffer"),
                contents: bytemuck::cast_slice(data),
                usage,
            })
    }

    pub fn update_buffer<T>(&self, buffer: &wgpu::Buffer, data: &[T])
    where
        T: bytemuck::Pod,
        T: bytemuck::Zeroable,
    {
        self.queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(data));
    }

    /// Creates a new [Mesh]
    ///
    /// Creates two [Buffer]s internally
    pub fn create_mesh(&self, vertices: &[Vertex], indices: &[u16]) -> Mesh {
        let vertex_buffer = self.create_buffer(vertices, wgpu::BufferUsages::VERTEX);
        let index_buffer = self.create_buffer(indices, wgpu::BufferUsages::INDEX);
        let model_buffer = self.create_buffer(
            &[ModelMatrices::new(
                Mat4::IDENTITY,
                Mat4::IDENTITY,
                Mat4::IDENTITY,
            )],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let model_bind_group = self.create_bind_group(
            &self.create_bind_group_layout(vec![BindGroupLayoutEntry::Buffer]),
            vec![model_buffer.as_entire_binding()],
        );

        Mesh {
            num_triangles: indices.len() as u32,
            vertex_buffer,
            index_buffer,
            model_buffer,
            model_bind_group,
        }
    }

    /// Creates a new [Shader]
    pub fn create_shader(
        &self,
        code: &'static str,
        bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
        config: SimplifiedPipelineConfig,
    ) -> Rc<Shader> {
        if let Some(shader) = self.shader_cache.borrow().get(&(code, config.clone())) {
            return shader.clone();
        }

        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(code.into()),
            });

        let pipeline = self.create_pipeline(&module, bind_group_layouts, config.clone());

        let shader = Rc::new(Shader { module, pipeline });
        self.shader_cache
            .borrow_mut()
            .insert((code, config), shader.clone());
        shader
    }

    pub fn create_render_texture(&self, size: UVec2) -> RenderTexture {
        RenderTexture {
            texture: Self::create_empty_texture(
                &self.device,
                size,
                self.surface.config.format,
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
            ),
            depth: Self::create_empty_texture(
                &self.device,
                size,
                wgpu::TextureFormat::Depth32Float,
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
            ),
        }
    }

    pub fn create_texture(&self, rgba8: &[u8], size: UVec2) -> Texture {
        let texture = Self::create_empty_texture(
            &self.device,
            size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            rgba8,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size.x),
                rows_per_image: std::num::NonZeroU32::new(size.y),
            },
            wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    pub fn create_bind_group_layout(
        &self,
        layout: Vec<BindGroupLayoutEntry>,
    ) -> wgpu::BindGroupLayout {
        let mut entries = Vec::<wgpu::BindGroupLayoutEntry>::new();

        for entry in layout {
            match entry {
                BindGroupLayoutEntry::Buffer => entries.push(wgpu::BindGroupLayoutEntry {
                    binding: entries.len() as u32,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }),

                BindGroupLayoutEntry::Sampler { filtering } => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: entries.len() as u32,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Sampler(if filtering {
                            wgpu::SamplerBindingType::Filtering
                        } else {
                            wgpu::SamplerBindingType::NonFiltering
                        }),
                        count: None,
                    })
                }

                BindGroupLayoutEntry::Texture { dim, filtering } => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: entries.len() as u32,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: (&dim).into(),
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: filtering,
                            },
                        },
                        count: None,
                    })
                }
            }
        }

        self.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
                entries: entries.as_slice(),
            })
    }

    pub fn create_bind_group(
        &self,
        layout: &wgpu::BindGroupLayout,
        bindings: Vec<wgpu::BindingResource>,
    ) -> wgpu::BindGroup {
        let mut entries = Vec::<wgpu::BindGroupEntry>::new();

        for resource in bindings {
            entries.push(wgpu::BindGroupEntry {
                binding: entries.len() as u32,
                resource,
            });
        }

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: entries.as_slice(),
            label: Some("Bind Group"),
        })
    }

    // internal functions

    /// Creates a new [wgpu::RenderPipeline]
    ///
    /// Quite generic pipeline creation. The only custom part is
    /// [SimplifiedPipelineConfig] which controls MSAA and wireframe. Cull
    /// mode is back by default, face culling ccw
    fn create_pipeline<'a>(
        &self,
        module: &wgpu::ShaderModule,
        bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
        config: SimplifiedPipelineConfig,
    ) -> wgpu::RenderPipeline {
        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: bind_group_layouts.as_slice(),
                push_constant_ranges: &[],
            });

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: "fs_main",
                    targets: config
                        .targets
                        .into_iter()
                        .map(|t| Some(t))
                        .collect::<Vec<Option<wgpu::ColorTargetState>>>()
                        .as_slice(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    // fixme: figure out what the right config for this is
                    //  gamebryo ships models with z-up, but we render with y-up
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: if config.wireframe {
                        wgpu::PolygonMode::Line
                    } else {
                        wgpu::PolygonMode::Fill
                    },
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: config.depth_settings.map(|ds| wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: ds.write,
                    depth_compare: ds.compare,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: match config.msaa {
                        MSAA::Off => 1,
                        MSAA::On(sc) => sc,
                    },
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    fn create_empty_texture(
        device: &wgpu::Device,
        size: UVec2,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Texture {
        let extend = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: extend,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            label: Some("Depth texture"),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler,
            dimensions: TextureDimensions::D2,
            size,
        }
    }

    fn create_surface_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Texture {
        Self::create_empty_texture(
            device,
            UVec2::new(config.width, config.height),
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }

    fn create_oit_opaque_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Texture {
        Self::create_empty_texture(
            device,
            UVec2::new(config.width, config.height),
            config.format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }

    fn create_oit_accum_texture(device: &wgpu::Device, size: UVec2) -> Texture {
        Self::create_empty_texture(
            device,
            size,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }

    fn create_oit_reveal_texture(device: &wgpu::Device, size: UVec2) -> Texture {
        Self::create_empty_texture(
            device,
            size,
            wgpu::TextureFormat::R8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }
}
