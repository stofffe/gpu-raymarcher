use wgpu::{
    util::DeviceExt, Adapter, BindGroup, Buffer, ComputePipeline, Device, PresentMode, Queue,
    RenderPipeline, Surface, SurfaceConfiguration,
};
use winit::window::Window;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

pub struct RenderContext {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) queue: wgpu::Queue,

    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) window_size: winit::dpi::PhysicalSize<u32>,
    pub(crate) window: Window,

    pub(crate) compute_pipeline: wgpu::ComputePipeline,
    pub(crate) readback_buffer: wgpu::Buffer,
    pub(crate) storage_buffer: wgpu::Buffer,
    pub(crate) compute_bind_group: wgpu::BindGroup,
    pub(crate) pixels_len: usize,

    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    width: u32,
    height: u32,
    test: f32,
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

        // Temp
        let globals = Globals {
            width: WIDTH,
            height: HEIGHT,
            test: 20.0,
        };
        let pixels = (0..32).collect::<Vec<u32>>();
        let (compute_pipeline, readback_buffer, storage_buffer, compute_bind_group) =
            create_compute_pipeline(&device, &pixels, globals);

        // Create render pipeline
        let render_pipeline = create_render_pipeline(&device, &surface_config);

        // Vertex and index buffer
        let (vertex_buffer, index_buffer, num_indices) = create_vertex_index_buffers(&device);

        let window_size = window.inner_size();

        let pixels_len = pixels.len();
        Self {
            window,
            surface,
            device,
            adapter,
            queue,

            surface_config,
            window_size,

            compute_pipeline,
            readback_buffer,
            storage_buffer,
            compute_bind_group,
            pixels_len,

            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
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

    fn execute_compute(&self) -> Vec<u32> {
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
            cpass.dispatch_workgroups(self.pixels_len as u32, 1, 1);
        }

        // Copy data from storage buffer to readback buffer
        let size = self.pixels_len * std::mem::size_of::<u32>();
        encoder.copy_buffer_to_buffer(
            &self.storage_buffer,
            0,
            &self.readback_buffer,
            0,
            size as wgpu::BufferAddress,
        );

        self.queue.submit(Some(encoder.finish()));

        // Read/Map readback buffer
        let buffer_slice = self.readback_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::MaintainBase::Wait);

        let data = buffer_slice.get_mapped_range();
        let mut result_u8 = bytemuck::cast_slice(&data).to_vec();
        let result_u32 = result_u8
            .chunks_exact_mut(4)
            .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
            .collect::<Vec<u32>>();

        // Need to unmap readback buffer
        drop(data);
        self.readback_buffer.unmap();

        println!("RESULT {:?}", result_u32);
        result_u32
    }

    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.execute_compute();

        // Render texture
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
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
    pixels: &[u32],
    globals: Globals,
) -> (ComputePipeline, Buffer, Buffer, BindGroup) {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("compute shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/compute_shader.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("compute bind group layout"),
        entries: &[
            // Pixel array
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
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
        ],
    });
    // TODO put in better place
    let size = (pixels.len() * std::mem::size_of::<u32>()) as wgpu::BufferAddress;

    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback buffer"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("storage buffer"),
        contents: bytemuck::cast_slice(pixels),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    // println!("{:?}", input);
    let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("global uniform buffer"),
        contents: bytemuck::cast_slice(&[globals]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("compute bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: global_uniform_buffer.as_entire_binding(),
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

    (pipeline, readback_buffer, storage_buffer, bind_group)
}

fn create_render_pipeline(
    device: &Device,
    surface_config: &SurfaceConfiguration,
) -> RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render_shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
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

    pipeline
}

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.8, 0.0] }, // bottom left
    Vertex { position: [1.0,  -1.0, 0.0] }, // bottom right
    Vertex { position: [-1.0, 1.0,  0.0] }, // top left
    Vertex { position: [1.0,  1.0,  0.0] }, // top right
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

/// Vertex representation
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}
