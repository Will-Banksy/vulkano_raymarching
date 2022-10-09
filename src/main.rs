use std::time::Instant;

use compute::Raymarch;
use minifb::{Window, WindowOptions, Key};

#[allow(unused)]
mod vulkan_computil;
mod compute;

// TODO List
// Shadows (do in shader ofc)
// Ability to control more of the rendering through the SceneInfo struct
// Also flesh out the mandelbulb rendering as different to a normal shape - Or at least, have the ability to pass out the iterations it took for colouring purposes
// UI to control rendering (do in something like imgui or just have another window running druid. Could actually display the fractal in the window running druid perhaps)
// Faster fractal rendering using vulkan fragment shaders or something (or if druid does it fast enough... I don't suppose it will though)

fn main() {
	let raymarch = Raymarch::new();

	let mut window = Window::new(
		"Raymarching - ESC To Exit (- fps)",
		1024 as usize,
		1024 as usize,
		WindowOptions::default()
	).unwrap();

	// Limit to ~20fps
	window.limit_update_rate(Some(std::time::Duration::from_secs_f32(0.05)));

	let mut delta_time = Instant::now();

	let mut frame_count = 0;

	while window.is_open() && !window.is_key_down(Key::Escape) {
		if window.is_key_down(Key::Up) || window.is_key_down(Key::Down) {
			let dir: f32 = if window.is_key_down(Key::Up) { 1. } else { -1. };
			let mut write_handle = raymarch._info_buffer.write().unwrap();
			write_handle.camera_pos[2] -= 5. * dir;
			write_handle.canvas_dist -= 5. * dir;
			println!("camera_pos: {}, canvas_dist: {}", write_handle.camera_pos[2], write_handle.canvas_dist);
		}

		if window.is_key_down(Key::W) || window.is_key_down(Key::S) {
			let dir: f32 = if window.is_key_down(Key::W) { 1. } else { -1. };
			raymarch._info_buffer.write().unwrap().camera_pos[2] -= 5. * dir;
		}

		let vk_buf = raymarch.render();
		let buf = vk_buf.read().unwrap();

		window.update_with_buffer(bytemuck::cast_slice(&buf[..]), 1024, 1024).unwrap();

		frame_count += 1;

		if frame_count == 10 {
			window.set_title(&format!("Raymarching - ESC To Exit ({} fps)", frame_count as f32 / delta_time.elapsed().as_secs_f32()));
			delta_time = Instant::now();
			frame_count = 0;
		}
	}



	let vk_buf = raymarch.debug_buffer;
	let buf = vk_buf.read().unwrap();
	println!("[DEBUG] ray_origin: {:?},\n[DEBUG] ray_direction: {:?}", buf.ray_origin, buf.ray_direction);
	// let img = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buf[..]).unwrap();
	// img.save("out.png").unwrap();
}
