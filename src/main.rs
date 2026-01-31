use fyrox::{
	engine::executor::Executor,
	renderer::framework::core::log::{Log, MessageKind},
};

mod koala_kombo;

fn main() {
	Log::set_verbosity(MessageKind::Warning);
	let mut executor = Executor::new();
	executor.add_plugin(koala_kombo::GamePlugin::default());
	executor.run();
}
