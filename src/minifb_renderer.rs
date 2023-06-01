use glm::{Vector3, vec3, cross, normalize, vec2, Vector2};
use minifb::{Window, WindowOptions, Key, MouseButton, MouseMode};
use std::{time::Instant, f32::consts::PI};

use crate::compute::Raymarch;

fn from_arr(a: &[f32; 3]) -> Vector3<f32> {
	vec3(a[0], a[1], a[2])
}

fn from_tup(a: &(f32, f32, f32)) -> Vector3<f32> {
	vec3(a.0, a.1, a.2)
}

fn from_vec(v: Vector3<f32>) -> [f32; 3] {
	[
		v.x,
		v.y,
		v.z
	]
}

// Returns (r, theta, phi) as per ISO standard
fn spherical(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
	let r = (x * x + y * y + z * z).sqrt();
	let theta = (z / r).asin();
	let phi = {
		if x > 0. {
			(y / x).atan()
		} else if x < 0. && y >= 0. {
			(y / x).atan() + PI
		} else if x < 0. && y < 0. {
			(y / x).atan() - PI
		} else if x == 0. && y > 0. {
			PI / 2.
		} else if x == 0. && y < 0. {
			-PI / 2.
		} else {
			f32::NAN
		}
	};
	(r, theta, phi)
}

// Returns (x, y, z)
fn cartesian(r: f32, theta: f32, phi: f32, zero_nan: bool) -> (f32, f32, f32) {
	let x = r * theta.cos() * if phi.cos().is_nan() && zero_nan { 1. } else { phi.cos() };
	let y = r * theta.cos() * if phi.sin().is_nan() && zero_nan { 1. } else { phi.sin() };
	let z = r * theta.sin();

	(
		if x.is_nan() && zero_nan { 0. } else { x },
		if y.is_nan() && zero_nan { 0. } else { y },
		z
	)
}

fn looparound(v: f32, lower: f32, upper: f32) -> f32 {
	if v < lower {
		upper + v - lower
	} else if v > upper {
		lower + v - upper
	} else {
		v
	}
}

fn to_camera_space(p: Vector2<f32>, cam_pos: Vector3<f32>, look_at: Vector3<f32>, canv_dist: f32) -> (f32, f32, f32) {
	let cam_dir = normalize(look_at - cam_pos);
	let uv_up_world = vec3(0., 1., 0.);
	let uv_right = normalize(cross(cam_dir, uv_up_world));
	let uv_down = normalize(cross(cam_dir, uv_right));
	let to_canvas = cam_dir * canv_dist;
	let i = (p / vec2(1024., 1024.)) * 2. - 1.;
	let p3d = cam_pos + to_canvas + (uv_right * i.x) + (uv_down * i.y);
	(p3d.x, p3d.y, p3d.z)
}

pub fn mkminifb() {
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

	const MOVE_AMT: f32 = 0.1;

	let mut prev_lmouse_down: bool = false;
	let mut prev_mouse_x: f32 = 0.;
	let mut prev_mouse_y: f32 = 0.;

	let mut speed_theta: f32 = 0.;
	let mut speed_phi: f32 = 0.;

	while window.is_open() && !window.is_key_down(Key::Escape) {
		// if window.is_key_down(Key::Up) || window.is_key_down(Key::Down) {
		// 	let dir: f32 = if window.is_key_down(Key::Up) { 1. } else { -1. };
		// 	let mut write_handle = raymarch._info_buffer.write().unwrap();
		// 	write_handle.camera_pos[2] -= MOVE_AMT * dir;
		// 	write_handle.canvas_dist -= MOVE_AMT * dir;
		// 	println!("camera_pos: {}, canvas_dist: {}", write_handle.camera_pos[2], write_handle.canvas_dist);
		// }

		if window.is_key_down(Key::W) || window.is_key_down(Key::S) {
			let dir: f32 = if window.is_key_down(Key::W) { 1. } else { -1. };
			// raymarch._info_buffer.write().unwrap().camera_pos[2] -= MOVE_AMT * dir;
			let mut write_handle = raymarch._info_buffer.write().unwrap();

			// Camera and target pos
			let mut cam_pos = from_arr(&write_handle.camera_pos);
			let mut tar_pos = from_arr(&write_handle.look_at);

			let cam_dir = normalize(tar_pos - cam_pos);
			let v_move = cam_dir * dir;

			cam_pos = cam_pos + v_move;
			tar_pos = tar_pos + v_move;

			write_handle.camera_pos = from_vec(cam_pos);
			write_handle.look_at = from_vec(tar_pos);
		}

		if window.is_key_down(Key::A) || window.is_key_down(Key::D) {
			let dir: f32 = if window.is_key_down(Key::A) { 1. } else { -1. };
			let mut write_handle = raymarch._info_buffer.write().unwrap();

			// Camera and target pos
			let mut cam_pos = from_arr(&write_handle.camera_pos);
			let mut tar_pos = from_arr(&write_handle.look_at);

			// println!("[BEFORE] cam_pos: {:?}, tar_pos: {:?}", cam_pos, tar_pos);

			let cam_dir = normalize(tar_pos - cam_pos);
			let uv_up_world = vec3(0., 1., 0.);
			let uv_move_dir = normalize(cross(uv_up_world, cam_dir)) * dir;
			let v_move = uv_move_dir * MOVE_AMT;

			cam_pos = cam_pos + v_move;
			tar_pos = tar_pos + v_move;

			write_handle.camera_pos = from_vec(cam_pos);
			write_handle.look_at = from_vec(tar_pos);
		}

		if window.is_key_down(Key::Space) || window.is_key_down(Key::LeftShift) {
			let dir: f32 = if window.is_key_down(Key::Space) { 1. } else { -1. };
			let mut write_handle = raymarch._info_buffer.write().unwrap();
			write_handle.camera_pos[1] -= MOVE_AMT * -dir;
			write_handle.look_at[1] -= MOVE_AMT * -dir;
		}

		{ // Do camera pointing
			let mut ib = raymarch._info_buffer.write().unwrap();

			let lmouse_down = window.get_mouse_down(MouseButton::Left);

			if prev_lmouse_down {
				if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
					// let cam_to_lookat = from_arr(&ib.look_at) - from_arr(&ib.camera_pos);

					// let (cm_r, cm_theta, cm_phi) = spherical(v.x, v.y, v.z);
					let (_, pm_theta, pm_phi): (f32, f32, f32) = {
						let m3d = to_camera_space(vec2(prev_mouse_x, prev_mouse_y), from_arr(&ib.camera_pos), from_arr(&ib.look_at), ib.canvas_dist);
						let cam_to_m3d = vec3(m3d.0, m3d.1, m3d.2) - from_arr(&ib.camera_pos);

						spherical(cam_to_m3d.x, cam_to_m3d.y, cam_to_m3d.z)
					};
					let (_, cm_theta, cm_phi): (f32, f32, f32) = {
						let m3d = to_camera_space(vec2(mx, my), from_arr(&ib.camera_pos), from_arr(&ib.look_at), ib.canvas_dist);
						let cam_to_m3d = vec3(m3d.0, m3d.1, m3d.2) - from_arr(&ib.camera_pos);

						spherical(cam_to_m3d.x, cam_to_m3d.y, cam_to_m3d.z)
					};

					// const DIVIDER: f32 = 100.;
					// let dx = mx - prev_mouse_x / DIVIDER;
					// let dy = my - prev_mouse_y / DIVIDER;

					speed_theta = pm_theta - cm_theta;
					speed_phi = pm_phi - cm_phi;
				}
			}

			// TODO: Find a way to rotate by speed_theta, speed_phi successfully
			if speed_theta != 0. || speed_phi != 0. { // Update camera pointing with speed
				let cam_to_lookat = from_arr(&ib.look_at) - from_arr(&ib.camera_pos);
				let (r, theta, phi) = spherical(cam_to_lookat.x, cam_to_lookat.y, cam_to_lookat.z);
				let (x, y, z) = cartesian(r, looparound(theta + speed_theta, 0., PI * 2.), looparound(phi + speed_phi, 0., PI * 2.), true);
				let v_nlookat = vec3(x, y, z) + from_arr(&ib.camera_pos);
				println!("[BEFORE] lookat: {:?}", ib.look_at);
				ib.look_at = [
					v_nlookat.x,
					v_nlookat.y,
					v_nlookat.z
				];
				println!("speed_theta: {}, speed_phi: {}", speed_theta, speed_phi);
				println!("[AFTER] lookat: {:?}", ib.look_at);
				speed_theta = 0.;
				speed_phi = 0.;
			}

			if lmouse_down != prev_lmouse_down {
				prev_lmouse_down = lmouse_down;
			}
			if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
				prev_mouse_x = mx;
				prev_mouse_y = my;
			}
		}

		let vk_buf = raymarch.render();
		let buf = vk_buf.read().unwrap();

		window.update_with_buffer(bytemuck::cast_slice(&buf[..]), 512, 512).unwrap();

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