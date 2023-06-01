mod renderer_widget;

use druid::{Widget, widget::{Flex, MainAxisAlignment, Label}, WidgetExt};

use crate::data::RendererData;

use self::renderer_widget::RendererWidget;

pub fn build_ui() -> impl Widget<RendererData> {
	Flex::row()
		.with_flex_child(
			Flex::column()
				.with_default_spacer()
				.with_child(Label::new("Renderer Window").with_text_size(32.))
				.with_flex_spacer(0.5)
				.with_child(RendererWidget::new())
				.with_flex_spacer(0.5)
				.expand_width(),
			1.0
		)
		.with_default_spacer()
		.with_child(
			Flex::column()
				.with_default_spacer()
				.with_child(Label::new("Control Panel").with_text_size(32.))
				.with_flex_spacer(1.0)
				.with_default_spacer()
		)
		.with_default_spacer()
		// .debug_paint_layout()
}