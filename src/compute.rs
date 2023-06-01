use std::sync::Arc;

use vulkano::{device::DeviceExtensions, buffer::{CpuAccessibleBuffer, BufferUsage}, image::{StorageImage, ImageDimensions, view::ImageView}, format::Format, pipeline::{ComputePipeline, Pipeline, PipelineBindPoint}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo}, sync::{self, GpuFuture}};

use crate::{vulkan_computil::{VkInstance, VkTarget, VK_QUEUEFLAGS_COMPUTE}};

use self::shaders::ray_marching_shader::ty::{SceneInfo, Shape, DebugInfo};

mod shaders {
	pub mod ray_marching_shader {
		vulkano_shaders::shader! {
			ty: "compute",
			path: "shaders/ray_marching_shader.comp",
			types_meta: {
				use bytemuck::{Zeroable, Pod};

				#[derive(Clone, Copy, Zeroable, Pod)]
			}
		}
	}
}

pub const RESULT_IMG_WIDTH: u32 = 512;
pub const RESULT_IMG_HEIGHT: u32 = 512;

#[allow(unused)]
const SHAPE_TYPE_NONE: u32 = 0;
#[allow(unused)]
const SHAPE_TYPE_SPHERE: u32 = 1;
#[allow(unused)]
const SHAPE_TYPE_WOBBLY_SPHERE: u32 = 2;
#[allow(unused)]
const SHAPE_TYPE_MANDELBULB: u32 = 3;

impl Default for Shape {
	fn default() -> Self {
		Self {
			position: Default::default(),
			size: Default::default(),
			albedo: Default::default(),
			shape_type: Default::default(),
			_dummy0: Default::default(),
			_dummy1: Default::default(),
		}
	}
}

pub struct Raymarch {
	_vk_instance: VkInstance,
	vk_target: VkTarget,
	pub _info_buffer: Arc<CpuAccessibleBuffer<SceneInfo>>,
	image: Arc<StorageImage>,
	output_buffer: Arc<CpuAccessibleBuffer<[u8]>>,
	pub debug_buffer: Arc<CpuAccessibleBuffer<DebugInfo>>,
	compute_pipeline: Arc<ComputePipeline>,
	descriptor_set: Arc<PersistentDescriptorSet>
}

impl Raymarch {
	pub fn new() -> Self {
		let vk_instance = VkInstance::new(None);

		let vk_target = VkTarget::new(vk_instance.clone(), VK_QUEUEFLAGS_COMPUTE, DeviceExtensions::default());

		let data: SceneInfo = SceneInfo {
			camera_pos: [0., 0., 10.],
			look_at: [0., 0., 0.],
			canvas_dist: 10.,
			num_shapes: 3,
			point_light: [0., 100., 200.],
			light_colour: [1.0, 1.0, 1.0],
			shapes: [
				Shape {
					position: [0., 0., 0.],
					size: [0.6, 0.6, 0.6],
					albedo: [0.1, 0.0, 0.2],
					shape_type: SHAPE_TYPE_MANDELBULB,
					_dummy0: [0; 4],
					_dummy1: [0; 4],
				},
				Shape {
					position: [1.6, 0., 0.],
					size: [1., 1., 1.],
					albedo: [0.0, 0.4, 0.8],
					shape_type: SHAPE_TYPE_SPHERE,
					_dummy0: [0; 4],
					_dummy1: [0; 4],
				},
				Shape {
					position: [-1.2, 0.5, -0.4],
					size: [1.80, 1.80, 1.80],
					albedo: [0.0, 1.0, 0.8],
					shape_type: SHAPE_TYPE_SPHERE,
					_dummy0: [0; 4],
					_dummy1: [0; 4],
				},
				{ Default::default() },
				{ Default::default() }, { Default::default() }, { Default::default() },
				{ Default::default() }, { Default::default() }, { Default::default() }
			],
			_dummy0: [0; 4],
			_dummy1: [0; 12],
			_dummy2: [0; 4],
    		_dummy3: [0; 4],
		};

		let info_buffer = CpuAccessibleBuffer::from_data(
			vk_target.device.clone(),
			BufferUsage { uniform_buffer: true, ..Default::default() },
			false,
			data
		).expect("Failed to create buffer");

		let image = StorageImage::new(vk_target.device.clone(),
			ImageDimensions::Dim2d {
				width: RESULT_IMG_WIDTH,
				height: RESULT_IMG_HEIGHT,
				array_layers: 1
			},
			Format::R8G8B8A8_UNORM,
			[vk_target.queue.queue_family_index()].into_iter()
		).expect("Failed to create storage image");
		let image_view = ImageView::new_default(image.clone()).unwrap();

		let output_buffer = CpuAccessibleBuffer::from_iter(
			vk_target.device.clone(),
			BufferUsage { transfer_dst: true, ..Default::default() },
			false,
			(0..(RESULT_IMG_WIDTH * RESULT_IMG_HEIGHT * 4)).map(|_| 0u8)
		).expect("Failed to create buffer");

		let debug_info: DebugInfo = DebugInfo {
			ray_origin: [0.; 3],
			ray_direction: [0.; 3],
			_dummy0: [0; 4],
		};

		let debug_buffer = CpuAccessibleBuffer::from_data(
			vk_target.device.clone(),
			BufferUsage { storage_buffer: true, ..Default::default() },
			false,
			debug_info
		).expect("Failed to create buffer");

		let shader = shaders::ray_marching_shader::load(vk_target.device.clone()).expect("Failed to load shader");

		let compute_pipeline = ComputePipeline::new(vk_target.device.clone(),
			shader.entry_point("main").unwrap(),
			&(), None, |_| {}
		).expect("Failed to create pipeline");

		let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
		let set = PersistentDescriptorSet::new(
			layout.clone(),
			[
				WriteDescriptorSet::buffer(0, info_buffer.clone()),
				WriteDescriptorSet::image_view(1, image_view),
				WriteDescriptorSet::buffer(2, debug_buffer.clone())
			]
		).unwrap();

		Self {
			_vk_instance: vk_instance,
			vk_target,
			_info_buffer: info_buffer,
			image,
			output_buffer,
			debug_buffer,
			compute_pipeline,
			descriptor_set: set
		}
	}

	pub fn render(&self) -> Arc<CpuAccessibleBuffer<[u8]>> {
		let mut builder = AutoCommandBufferBuilder::primary(
			self.vk_target.device.clone(),
			self.vk_target.queue.queue_family_index(),
			CommandBufferUsage::MultipleSubmit
		).unwrap();

		builder
			.bind_pipeline_compute(self.compute_pipeline.clone())
			.bind_descriptor_sets(PipelineBindPoint::Compute,
				self.compute_pipeline.layout().clone(),
				0, self.descriptor_set.clone()
			)
			.dispatch([RESULT_IMG_WIDTH / 8, RESULT_IMG_HEIGHT / 8, 1])
			.unwrap()
			.copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
				self.image.clone(),
				self.output_buffer.clone()
			))
			.unwrap();

		let command_buffer = builder.build().unwrap();

		let future = sync::now(self.vk_target.device.clone())
			.then_execute(self.vk_target.queue.clone(), command_buffer)
			.unwrap()
			.then_signal_fence_and_flush()
			.unwrap();

		future.wait(None).unwrap();

		self.output_buffer.clone()
	}
}