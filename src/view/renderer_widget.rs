use druid::{Widget, Size, RenderContext, piet::{ImageFormat, InterpolationMode}, Data};

use crate::{compute::{Raymarch, self}, data::RendererData};

pub struct RendererWidget {
	engine: Raymarch,
	img_data: Option<Vec<u8>>
}

impl RendererWidget {
	pub fn new() -> Self {
		RendererWidget { engine: Raymarch::new(), img_data: None }
	}
}

#[allow(unused)]
impl Widget<RendererData> for RendererWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut RendererData, env: &druid::Env) {
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &RendererData, env: &druid::Env) {
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &RendererData, data: &RendererData, env: &druid::Env) {
		if data.same(old_data) {
			self.img_data.take();
		}
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &RendererData, env: &druid::Env) -> druid::Size {
		Size::new(compute::RESULT_IMG_WIDTH as f64, compute::RESULT_IMG_HEIGHT as f64)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &RendererData, env: &druid::Env) {
		if let None = self.img_data {
			let buf = self.engine.render();
			self.img_data = Some(buf.read().unwrap().to_vec());
		}

		let img = ctx.make_image(compute::RESULT_IMG_WIDTH as usize, compute::RESULT_IMG_HEIGHT as usize, &self.img_data.as_ref().unwrap(), ImageFormat::RgbaSeparate).unwrap();
		let rect = ctx.size().to_rect();
		ctx.draw_image(&img, rect, InterpolationMode::NearestNeighbor);
    }
}