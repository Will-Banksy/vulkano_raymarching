use std::sync::Arc;

use vulkano::{instance::{Instance, InstanceExtensions, InstanceCreateInfo}, device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, Queue, DeviceExtensions, DeviceCreateInfo, QueueCreateInfo, QueueFamilyProperties, QueueFlags}, VulkanLibrary, sync::PipelineStage, swapchain::Surface};

pub const VK_QUEUEFLAGS_COMPUTE: QueueFlags = QueueFlags { compute: true, ..QueueFlags::empty() };

#[derive(Clone)]
pub struct VkInstance {
	pub instance: Arc<Instance>
}

pub struct VkTarget {
	pub physical: Arc<PhysicalDevice>,
	pub device: Arc<Device>,
	pub queue: Arc<Queue>,
}

pub struct VkComputePipeline { // TODO: Write a VkComputePipeline class
}

impl VkInstance {
	pub fn new(ext: Option<InstanceExtensions>) -> Self {
		let library = VulkanLibrary::new().unwrap();

		let extensions = ext.unwrap_or(InstanceExtensions {
			..Default::default()
		});

		VkInstance {
			instance: Instance::new(library, InstanceCreateInfo {
				enabled_extensions: extensions,
				..InstanceCreateInfo::application_from_cargo_toml()
			}).expect("Failed to create Vulkan instance")
		}
	}
}

impl VkTarget {
	/// Attempts to find the best Vulkan implementation available and the best QueueFamilies/Queues
	pub fn new(vk_instance: VkInstance, queue_flags: QueueFlags, device_extensions: DeviceExtensions) -> Self {
		let (physical, queue_family_index) = VkTarget::select_compute_device(vk_instance.instance, &device_extensions, queue_flags);

		println!("Using physical vulkan device: {} (type: {:?})", physical.properties().device_name, physical.properties().device_type);

		let (device, mut queues) = Device::new(
			physical.clone(),
			DeviceCreateInfo {
				enabled_extensions: device_extensions,
				queue_create_infos: vec![QueueCreateInfo {
					queue_family_index,
					..Default::default()
				}],
				..Default::default()
			}
		).expect("Failed to create a device");

		let queue = queues.next().unwrap();

		VkTarget {
			physical,
			device,
			queue
		}
	}

	// Attempts to find the best Vulkan implementation and QueueFamily (returned as an index)
	pub fn select_compute_device(instance: Arc<Instance>, device_extensions: &DeviceExtensions, queue_flags: QueueFlags) -> (Arc<PhysicalDevice>, u32) {
		instance.enumerate_physical_devices().expect("Cannot enumerate physical devices")
			.filter(|p| p.supported_extensions().contains(&device_extensions))
			.filter_map(|p| {
				// The Vulkan specs guarantee that a compliant implementation must provide at least one queue that supports compute operations
				p.queue_family_properties().iter().enumerate()
					.position(|(_, q)| {
						q.queue_flags.contains(&queue_flags)
					})
					.map(|i| (p.clone(), i as u32))
			})
			.min_by_key(|(p, _)| match p.properties().device_type { // Order by device type. Preferably we want to use a discrete gpu
				PhysicalDeviceType::DiscreteGpu => 0,
				PhysicalDeviceType::IntegratedGpu => 1,
				PhysicalDeviceType::VirtualGpu => 2,
				PhysicalDeviceType::Cpu => 3,
				PhysicalDeviceType::Other => 4,
				_ => 5
			}).expect("No vulkan implementations found")
	}
}
