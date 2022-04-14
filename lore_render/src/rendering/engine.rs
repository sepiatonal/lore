use crate::{
    mesh::{
        Mesh, Vertex,
    },
    rendering::camera::RenderableCamera,
};
use std::{
    mem::size_of,
    str,
};
use cgmath::{
    prelude::*,
    *,
};
use slab::Slab;
use wgpu::{
    *,
    util::{
        BufferInitDescriptor, DeviceExt, StagingBelt,
    },
};
use winit::{
    window::Window,
    dpi::PhysicalSize,
};
use wgpu_glyph::GlyphBrush;
use image::GenericImageView;
use std::io::Read;
use futures::task::SpawnExt;

pub struct RenderingInstance {
    // TODO oh god, please split this into multiple files you idiot
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,

    text_staging_belt: StagingBelt,
    local_pool: futures::executor::LocalPool,
    local_spawner: futures::executor::LocalSpawner,

    render_pipeline_layout: PipelineLayout,
    texture_bind_group_layout: BindGroupLayout,

    render_pipelines: Slab<RenderPipeline>,
    loaded_meshes: Slab<LoadedMesh>,
    textures: Slab<Texture>,
    glyph_brushes: Slab<GlyphBrush<()>>,
    text_instances: Slab<TextInstance>,
    camera: RenderableCamera,
}

impl RenderingInstance {
    pub(crate) async fn new(window: &Window) -> Self {
        // Most of this is self-explanatory, but to the extent that it's not,
        // look up the wgpu-rs book tutorial. It's not too divergent from how
        // things are done there.
        let size = window.inner_size();
        let instance = wgpu::Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.expect("Failed to create WGPU adapter.");
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: Limits::default(),
                label: None,
            },
            None
        ).await.expect("Failed to create WGPU Device.");
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let camera = RenderableCamera::new(&device);
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let mut ret = Self {
            surface,
            device,
            queue,
            config,
            size,

            text_staging_belt: staging_belt,
            local_pool,
            local_spawner,

            render_pipeline_layout,
            texture_bind_group_layout,
            render_pipelines: Slab::new(),
            loaded_meshes: Slab::new(),
            textures: Slab::new(),
            glyph_brushes: Slab::new(),
            text_instances: Slab::new(),
            camera,
        };

        ret.create_texture(crate::asset_loading::images::default_texture());

        ret
    }

    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        } // TODO else panic?
    }

    pub(crate) fn refresh_surface_configuration(&mut self) {
        self.config.width = self.size.width;
        self.config.height = self.size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn create_render_pipeline(&mut self, shader_src: &str) -> usize {
        let shader = self.create_shader_module(shader_src);
        let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor{
            label: None,
            layout: Some(&self.render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                    RawObjectInstance::desc(),
                ],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: self.config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }]
            }),
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
        });
        self.render_pipelines.insert(pipeline)
    }

    fn create_shader_module(&mut self, shader_src: &str) -> ShaderModule {
        self.device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(shader_src.into()),
        })
    }

    pub fn create_default_render_pipeline(&mut self) -> usize {
        self.create_render_pipeline(include_str!("../../shaders/default_shader.wgsl"))
    } // TODO this function belongs elsewhere, somewhere closer to the API level instead of backend

    pub fn create_default_gui_render_pipeline(&mut self) -> usize {
        self.create_render_pipeline(include_str!("../../shaders/gui_shader.wgsl"))
    } // TODO this function belongs elsewhere, somewhere closer to the API level instead of backend

    pub fn bind_mesh(&mut self, mesh: &Mesh, render_pipeline: usize, texture_id: Option<usize>) -> usize {
        let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: BufferUsages::INDEX,
        });
        let instance_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &[],
            usage: BufferUsages::VERTEX,
        });

        self.loaded_meshes.insert(
        LoadedMesh {
                vertex_buffer,
                index_buffer,
                num_indices: mesh.indices.len() as u32,
                render_pipeline,
                instances: Slab::with_capacity(5),
                instance_buffer,
                texture_id,
            }
        )
    }

    pub fn create_object_instance(&mut self, mesh: usize, instance: ObjectInstance) -> (usize, usize) {
        (mesh, self.loaded_meshes[mesh].add_instance(instance))
    }

    pub fn remove_object_instance(&mut self, instance_id: (usize, usize)) {
        let (mesh, instance) = instance_id;
        self.loaded_meshes[mesh].remove_instance(instance);
    }

    pub fn modify_instance(&mut self, instance_id: (usize, usize), fun: fn(&mut ObjectInstance) -> ()) {
        let (mesh, instance) = instance_id;
        let mut instance_obj = self.loaded_meshes[mesh].instances.get_mut(instance).unwrap();
        fun(&mut instance_obj);
    }

    pub fn get_instance_mut(&mut self, instance_id: (usize, usize)) -> &mut ObjectInstance {
        let (mesh, instance) = instance_id;
        self.loaded_meshes[mesh].instances.get_mut(instance).unwrap()
    }

    pub fn read_instance(&mut self, instance_id: (usize, usize)) -> &ObjectInstance {
        let (mesh, instance) = instance_id;
        let instance_obj = self.loaded_meshes[mesh].instances.get(instance).unwrap();
        return instance_obj;
    }

    pub fn set_camera_transform(&mut self, new_position: Option<Point3<f32>>, new_target: Option<Point3<f32>>, new_up: Option<Vector3<f32>>) {
        if let Some(pos) = new_position {
            self.camera.camera.pos = pos;
        }
        if let Some(target) = new_target {
            self.camera.camera.target = target;
        }
        if let Some(up) = new_up {
            self.camera.camera.up = up;
        }
    }

    pub fn create_texture(&mut self, img: image::DynamicImage) -> usize {
        let imgbuf = img.to_rgba();
        let (width, height) = img.dimensions();
        let tex_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let gpu_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
        });
        self.queue.write_texture(
            wgpu::ImageCopyTextureBase { 
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &imgbuf,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            tex_size,
        );
        let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: None,
            }
        );
        self.textures.insert(Texture { bind_group })
    }

    pub fn create_glyph_brush(&mut self, font_path: &str) -> usize {
        let file = std::fs::File::open(font_path).unwrap();
        let mut file_reader = std::io::BufReader::new(file);
        let mut file_buffer = Vec::new();
        file_reader.read_to_end(&mut file_buffer).unwrap();
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_vec(file_buffer).unwrap();
        let brush = wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&mut self.device, self.config.format);
        self.glyph_brushes.insert(brush)
    }

    pub fn create_text_box(&mut self, text_instance: TextInstance) -> usize {
        self.text_instances.insert(text_instance)
    }

    pub fn delete_text_box(&mut self, text_instance: usize) {
        self.text_instances.remove(text_instance);
    }

    pub fn get_textbox_mut(&mut self, text_instance: usize) -> &mut TextInstance {
        self.text_instances.get_mut(text_instance).unwrap()
    }

    // TODO implement delete_shader_program
    pub fn delete_shader_program() {}

    // TODO implement delete_mesh
    pub fn delete_mesh() {}

    // TODO implement delete_texture
    pub fn delete_texture() {}

    pub(crate) fn update(&mut self) {
        self.camera.update(&mut self.queue);
        for (_, mesh) in self.loaded_meshes.iter_mut() {
            mesh.update_instance_buffer(&mut self.device);
        }
    }

    // TODO this function cannot take mut self, it must be &self
    pub(crate) fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        { // scope block to release the borrow of encoder when done
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
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
                }],
                depth_stencil_attachment: None,
            });

            // TODO as noted in the documentation, .iter() on a Slab is SLOW
            // but we can make it not slow by regularly comapcting the Slab.
            // so TODO: implement packing/defraging of loaded_meshes at regular intervals,
            // or find a different data structure to use.
            for (_, m) in self.loaded_meshes.iter() {
                render_pass.set_pipeline(&self.render_pipelines[m.render_pipeline]);
                if let Some(id) = m.texture_id {
                    render_pass.set_bind_group(0, &self.textures[id].bind_group, &[]);
                } else {
                    render_pass.set_bind_group(0, &self.textures[0].bind_group, &[]);
                }
                render_pass.set_bind_group(1, &self.camera.bind_group, &[]);
                render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, m.instance_buffer.slice(..));
                render_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..m.num_indices, 0, 0..(m.instances.len() as u32));
            }
        }
        for (_, txt) in self.text_instances.iter() {
            let brush = self.glyph_brushes.get_mut(txt.brush).unwrap();
            brush.queue_custom_layout(
                wgpu_glyph::Section {
                    screen_position: txt.position,
                    bounds: txt.dimensions,
                    text: vec!(wgpu_glyph::Text::new(&txt.text).with_color(txt.color).with_scale(txt.scale)),
                    ..wgpu_glyph::Section::default()
                },
                &wgpu_glyph::Layout::default_wrap()
                    .h_align(wgpu_glyph::HorizontalAlign::Center)
                    .v_align(wgpu_glyph::VerticalAlign::Top),
            );
            brush.draw_queued(
                &self.device,
                &mut self.text_staging_belt,
                &mut encoder,
                &view,
                640,
                480,
            ).unwrap();
        }
        
        self.text_staging_belt.finish();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.local_spawner.spawn(self.text_staging_belt.recall()).unwrap();
        self.local_pool.run_until_stalled();

        Ok(())
    }
}

/// A mesh with loaded vertex/index buffers
struct LoadedMesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    render_pipeline: usize,
    // TODO instances should be stored in their own place that references the mesh,
    // but then we'll have to write some nice code to efficiently construct the
    // instances range every frame.
    instances: Slab<ObjectInstance>,
    instance_buffer: Buffer,
    texture_id: Option<usize>,
}

pub struct ObjectInstance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl ObjectInstance {
    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::<f32>::new(x, y, z),
            rotation: Quaternion::<f32>::from_axis_angle(Vector3::unit_z(), Deg(0.0)),
        }
    }

    pub fn with_angle(mut self, axis: Vector3<f32>, angle_deg: f32) -> Self {
        self.rotation = Quaternion::<f32>::from_axis_angle(axis, Deg(angle_deg));
        self
    }

    fn as_raw(&self) -> RawObjectInstance {
        let mat4 = Matrix4::from_translation(self.position) * Matrix4::from(self.rotation);
        RawObjectInstance { matrix: mat4.into() }
    }
}

impl LoadedMesh {
    fn add_instance(&mut self, instance: ObjectInstance) -> usize {
        self.instances.insert(instance)
    }

    fn remove_instance(&mut self, instance: usize) {
        self.instances.remove(instance);
    }

    pub fn update_instance_buffer(&mut self, device: &mut Device) {
        let data = self.instances.iter().map(|(_, inst)| inst.as_raw().matrix).collect::<Vec<_>>();
        self.instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
    }
}

struct Texture {
    bind_group: BindGroup,
}

#[repr(C)]
struct RawObjectInstance {
    matrix: [[f32;4];4],
}

impl RawObjectInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RawObjectInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct TextInstance {
    pub position: (f32, f32),
    pub dimensions: (f32, f32),
    pub color: [f32; 4],
    pub scale: f32,
    pub text: String,
    pub brush: usize,
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: (size_of::<[f32;3]>() as BufferAddress) * 2,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                }
            ]
        }
    }
}