#[allow(unused)]
mod vulkan_computil;
mod compute;
mod view;
mod data;
mod delegate;
mod minifb_renderer;

// TODO List
// Camera control
// Shadows (do in shader ofc)
// Ability to control more of the rendering through the SceneInfo struct
// Also flesh out the mandelbulb rendering as different to a normal shape - Or at least, have the ability to pass out the iterations it took for colouring purposes
// UI to control rendering (do in something like imgui or just have another window running druid. Could actually display the fractal in the window running druid perhaps)
// Faster fractal rendering using vulkan fragment shaders or something (or if druid does it fast enough... I don't suppose it will though)

use data::RendererData;
use delegate::Delegate;
use druid::{AppLauncher, WindowDesc, PlatformError};
use view::build_ui;

fn main() -> Result<(), PlatformError> {
	minifb_renderer::mkminifb();
	let data = RendererData {};

	AppLauncher::with_window(WindowDesc::new(build_ui())
		.title("Raymarching Renderer")
		.window_size((1024., 800.))
	)
		.delegate(Delegate {})
		.launch(data)?;

	Ok(())
}
