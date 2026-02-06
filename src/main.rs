use fyrox::{
	dpi::LogicalSize,
	engine::{GraphicsContextParams, executor::Executor},
	event_loop::EventLoop,
	renderer::framework::core::log::{Log, MessageKind},
	window::WindowAttributes,
};

mod koala_kombo;
mod plugin;

fn main() {
	Log::set_verbosity(MessageKind::Warning);

	let event_loop = EventLoop::new().unwrap();

	let mut window_attributes = WindowAttributes::default();
	window_attributes.title = String::from("Koala Kombo");
	window_attributes.resizable = false;
	window_attributes.inner_size = Some(LogicalSize::new(500.0, 650.0).into());

	let params = GraphicsContextParams {
		window_attributes,
		vsync: true,
		msaa_sample_count: None,
		graphics_server_constructor: Default::default(),
	};

	let mut executor = Executor::from_params(event_loop, params);
	executor.add_plugin(plugin::GamePlugin::default());
	executor.run();
}
