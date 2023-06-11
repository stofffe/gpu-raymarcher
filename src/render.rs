use encase::{ShaderType, StorageBuffer, UniformBuffer};
use glam::Mat3;
use glam::{uvec2, vec3, UVec2, Vec3};
use wgpu::{
    util::DeviceExt, Adapter, BindGroup, Buffer, ComputePipeline, Device, Extent3d, PresentMode,
    Queue, RenderPipeline, Surface, SurfaceConfiguration, TextureView,
};
use winit::window::Window;

use crate::time::TimeContext;

pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 720;
pub const MAX_SHAPE_AMOUNT: u64 = 256;

pub struct RenderContext {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) queue: wgpu::Queue,

    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) window_size: winit::dpi::PhysicalSize<u32>,
    pub(crate) window: Window,

    pub(crate) compute_pipeline: wgpu::ComputePipeline,
    pub(crate) compute_bind_group: wgpu::BindGroup,
    // These two are a part of the bind group
    pub(crate) input_buffer: wgpu::Buffer,
    pub(crate) global_uniform_buffer: wgpu::Buffer,
    pub(crate) texture_view: wgpu::TextureView,

    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
    pub(crate) texture_bind_group: wgpu::BindGroup,

    pub(crate) globals: Globals,
    pub(crate) resolution: (u32, u32),
    pub(crate) shapes: Vec<Shape>,
    // pub(crate) shapes: Shapes,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Sphere {
        pos: Vec3,
        radius: f32,
    },
    BoxExact {
        pos: Vec3,
        b: Vec3,
    },
    Plane {
        pos: Vec3,
        normal: Vec3,
    },
    Union {
        shape1: Box<Shape>,
        shape2: Box<Shape>,
    },
    Intersection {
        shape1: Box<Shape>,
        shape2: Box<Shape>,
    },
    Subtraction {
        shape1: Box<Shape>,
        shape2: Box<Shape>,
    },
}

pub fn shapes_to_gpu(shapes: &Vec<Shape>) -> ShapesGPU {
    let mut gpu_shapes = ShapesGPU(Vec::new());
    for shape in shapes.iter() {
        gpu_shapes.add(shape);
    }
    gpu_shapes
}

#[derive(Default, Debug, Clone, ShaderType)]
pub struct ShapeGPU {
    pub pos: Vec3,
    pub id: u32,
    pub v1: Vec3,
    pub f1: f32,
}

#[derive(Debug, Clone)]
pub struct ShapesGPU(Vec<ShapeGPU>);

impl ShapesGPU {
    pub fn add(&mut self, shape: &Shape) {
        match shape {
            Shape::Union { shape1, shape2 } => {
                self.0.push(ShapeGPU {
                    id: 0,
                    ..Default::default()
                });
                self.add(shape1);
                self.add(shape2);
            }
            Shape::Intersection { shape1, shape2 } => {
                self.0.push(ShapeGPU {
                    id: 1,
                    ..Default::default()
                });
                self.add(shape1);
                self.add(shape2);
            }
            Shape::Subtraction { shape1, shape2 } => {
                self.0.push(ShapeGPU {
                    id: 2,
                    ..Default::default()
                });
                self.add(shape1);
                self.add(shape2);
            }
            Shape::Sphere { pos, radius } => self.0.push(ShapeGPU {
                id: 6,
                pos: *pos,
                f1: *radius,
                ..Default::default()
            }),
            Shape::BoxExact { pos, b } => self.0.push(ShapeGPU {
                pos: *pos,
                id: 7,
                v1: *b,
                ..Default::default()
            }),
            Shape::Plane { pos, normal } => self.0.push(ShapeGPU {
                pos: *pos,
                id: 8,
                v1: *normal,
                ..Default::default()
            }),
        };
    }
}

// ShaderType auto pads!
// Try to minimize size
#[derive(Debug, Clone, ShaderType)]
pub(crate) struct Globals {
    pub(crate) screen_dim: UVec2,
    pub(crate) camera_pos: Vec3,
    pub(crate) camera_rot: Mat3,
    pub(crate) light_pos: Vec3,
    pub(crate) focal_length: f32,
    pub(crate) time: f32,
    pub(crate) shape_amount: u32,
}
impl RenderContext {
    // Creating some of the wgpu types requires async code
    pub(crate) async fn new(window: Window) -> Self {
        // Init wpgu
        let (surface, adapter, device, queue) = init_wpgu(&window).await;

        // Configure surface
        let surface_config =
            create_surface_config(&window, &surface, &adapter, PresentMode::AutoVsync);
        surface.configure(&device, &surface_config);

        // Default global data
        let globals = Globals {
            camera_pos: Vec3::ZERO,
            camera_rot: Mat3::from_rotation_y(0.0),
            light_pos: vec3(-2.0, 2.0, -4.0),
            screen_dim: uvec2(WIDTH, HEIGHT),
            focal_length: 1.0,
            time: 2.0,
            shape_amount: 0,
        };
        dbg!(Globals::min_size());
        dbg!(ShapeGPU::min_size());

        // let spheres = Vec::<ShapeGPU>::with_capacity(MAX_SHAPE_AMOUNT as usize);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture desc"),
            size: Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create compute pipeline
        let (compute_pipeline, input_buffer, global_uniform_buffer, compute_bind_group) =
            create_compute_pipeline(&device, &globals, &texture_view);

        // Create render pipeline
        let (render_pipeline, texture_bind_group) =
            create_render_pipeline(&device, &surface_config, &texture_view);

        // Vertex and index buffer
        let (vertex_buffer, index_buffer, num_indices) = create_vertex_index_buffers(&device);

        let window_size = window.inner_size();

        let shapes = Vec::with_capacity(MAX_SHAPE_AMOUNT as usize);

        Self {
            window,
            surface,
            device,
            adapter,
            queue,

            surface_config,
            window_size,

            compute_pipeline,
            input_buffer,
            global_uniform_buffer,
            compute_bind_group,
            texture_view,

            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            texture_bind_group,

            globals,
            resolution: (WIDTH, HEIGHT),
            shapes,
        }
    }

    pub(crate) fn reconfigure_present_mode(&mut self, present_mode: PresentMode) {
        self.surface_config.present_mode = present_mode;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub(crate) fn resize_window(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    fn execute_raymarch(&mut self, time_ctx: &TimeContext) {
        self.update_global_uniforms(time_ctx, self.shapes.len() as u32);
        self.update_input_buffer(shapes_to_gpu(&self.shapes));
        self.execute_compute();
        self.shapes.clear();
    }

    fn update_global_uniforms(&mut self, time_ctx: &TimeContext, len: u32) {
        // Update fields
        self.globals.time = time_ctx.time_since_start();
        self.globals.shape_amount = len;

        // Update buffer
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self.globals).unwrap();
        let byte_buffer = buffer.into_inner();
        self.queue
            .write_buffer(&self.global_uniform_buffer, 0, &byte_buffer);
    }

    fn update_input_buffer(&mut self, shapes: ShapesGPU) {
        // dbg!(&shapes);
        // self.spheres[0].pos += vec3(0.0, 0.1, 0.0);
        let mut byte_buffer = Vec::new();
        let mut buffer = StorageBuffer::new(&mut byte_buffer);
        buffer.write(&shapes.0).unwrap();
        self.queue.write_buffer(&self.input_buffer, 0, &byte_buffer);
    }

    fn execute_compute(&mut self) {
        // Execute compute pass
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("compute encoder"),
            });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute pass"),
            });
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.dispatch_workgroups(WIDTH, HEIGHT, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    pub(crate) fn render(&mut self, time_ctx: &TimeContext) -> Result<(), wgpu::SurfaceError> {
        // Execute raymarching compute shader
        self.execute_raymarch(time_ctx);

        // Render texture;
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

async fn init_wpgu(window: &Window) -> (Surface, Adapter, Device, Queue) {
    // Create surface
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    // Create adapter. device and queue
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        )
        .await
        .unwrap();
    (surface, adapter, device, queue)
}

fn create_surface_config(
    window: &Window,
    surface: &Surface,
    adapter: &Adapter,
    present_mode: PresentMode,
) -> SurfaceConfiguration {
    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format: wgpu::TextureFormat = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.describe().srgb)
        .unwrap_or(surface_caps.formats[0]);
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        // present_mode: surface_caps.present_modes[0],
        // present_mode: PresentMode::AutoVsync,
        present_mode,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    }
}

fn create_compute_pipeline(
    device: &Device,
    globals: &Globals,
    texture_view: &TextureView,
) -> (ComputePipeline, Buffer, Buffer, BindGroup) {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("compute shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/compute_shader.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("compute bind group layout"),
        entries: &[
            // Input array
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true }, // read only
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    // min_binding_size: Some(NonZeroU64::new(1).unwrap()),
                },
                count: None,
            },
            // Globals
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            },
            // Texture
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm, // TODO SRGB?
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });

    let buffer_size = u64::from(ShapeGPU::min_size()) * MAX_SHAPE_AMOUNT;
    let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("shape buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Globals unfiform
    let mut buffer = UniformBuffer::new(Vec::new());
    buffer.write(&globals).unwrap();
    let byte_buffer = buffer.into_inner();

    let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("global uniform buffer"),
        contents: &byte_buffer,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        // contents: bytemuck::cast_slice(&[globals]),
    });

    // Bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("compute bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: global_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("compute pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("compute pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: "cs_main",
    });

    (pipeline, input_buffer, global_uniform_buffer, bind_group)
}

fn create_render_pipeline(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    texture_view: &TextureView,
) -> (RenderPipeline, BindGroup) {
    let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ],
        label: Some("diffuse bind group"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render_shader.wgsl").into()),
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    (pipeline, texture_bind_group)
}

/// Vertex representation
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 1.0]}, // bottom left 
    Vertex { position: [1.0,  -1.0, 0.0], uv: [1.0, 1.0]}, // bottom right
    Vertex { position: [-1.0, 1.0,  0.0], uv: [0.0, 0.0]}, // top left
    Vertex { position: [1.0,  1.0,  0.0], uv: [1.0, 0.0]}, // top right
];
const INDICES: &[u16] = &[0, 1, 2, 3, 2, 1];

fn create_vertex_index_buffers(device: &Device) -> (Buffer, Buffer, u32) {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let num_indices = INDICES.len() as u32;

    (vertex_buffer, index_buffer, num_indices)
}
