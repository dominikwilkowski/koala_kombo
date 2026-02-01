use fyrox::{
	engine::executor::Executor,
	renderer::framework::core::log::{Log, MessageKind},
};

mod koala_kombo;
mod plugin;

fn main() {
	Log::set_verbosity(MessageKind::Warning);
	let mut executor = Executor::new();
	executor.add_plugin(plugin::GamePlugin::default());
	executor.run();
}
