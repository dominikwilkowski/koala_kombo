use fyrox::engine::executor::Executor;

mod block_blast_ui;

fn main() {
	let mut executor = Executor::new();
	executor.add_plugin(block_blast_ui::GamePlugin::default());
	executor.run();
}
