use fyrox::engine::executor::Executor;

mod koala_kombo;

fn main() {
	let mut executor = Executor::new();
	executor.add_plugin(koala_kombo::GamePlugin::default());
	executor.run();
}
