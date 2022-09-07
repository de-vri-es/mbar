use std::sync::Arc;
use std::time::Duration;

use glutin::platform::unix::WindowBuilderExtUnix;
use glutin::platform::run_return::EventLoopExtRunReturn;

use egui_glow::egui_winit::egui;

#[derive(clap::Parser)]
struct Options {
	#[clap(short, long)]
	screen: Option<usize>,
}

fn main() {
	env_logger::init_from_env("RUST_LOG");
	if let Err(()) = do_main(clap::Parser::parse()) {
		std::process::exit(1);
	}
}

fn do_main(options: Options) -> Result<(), ()> {
	let mut event_loop = glutin::event_loop::EventLoopBuilder::new().build();
	let mut window = Window::new(&event_loop, "mbar")?;

	let screen = match options.screen {
		None => {
			event_loop.primary_monitor()
				.ok_or_else(|| log::error!("Failed to detect primary monitor."))?
		}
		Some(screen) => {
			event_loop.available_monitors()
				.nth(screen)
				.ok_or_else(|| log::error!("Invalid screen index: {}, only {} screens available", screen, event_loop.available_monitors().count()))?
		},
	};
	let position = screen.position();
	let mut size = screen.size();
	size.height = 300;

	window.native().set_outer_position(position);
	window.native().set_inner_size(size);
	window.native().set_visible(true);

	let exit_code = event_loop.run_return(|event, _event_loop, control_flow| {
		match event {
			glutin::event::Event::LoopDestroyed => {
				window.ui.destroy();
			},
			glutin::event::Event::RedrawRequested(_window_id) => {
				let repaint_after = window.run();
				if repaint_after.is_zero() {
					window.window.window().request_redraw();
				} else if let Some(wait_until) = std::time::Instant::now().checked_add(repaint_after) {
					control_flow.set_wait_until(wait_until);
				} else {
					control_flow.set_wait();
				}

				unsafe {
					use glow::HasContext;
					window.gl.clear_color(1.0, 0.0, 1.0, 1.0);
					window.gl.clear(glow::COLOR_BUFFER_BIT);
				}

				window.ui.paint(window.window.window());
				if let Err(e) = window.window.swap_buffers() {
					log::error!("Failed to swap window buffers: {}", e);
				}
			},
			glutin::event::Event::NewEvents(glutin::event::StartCause::ResumeTimeReached { .. }) => {
				window.window.window().request_redraw();
			},
			glutin::event::Event::WindowEvent { window_id: _, event } => {
				match &event {
					glutin::event::WindowEvent::CloseRequested | glutin::event::WindowEvent::Destroyed => {
						control_flow.set_exit();
					},
					glutin::event::WindowEvent::Resized(size) => {
						window.window.resize(*size);
					},
					glutin::event::WindowEvent::ScaleFactorChanged { scale_factor: _, new_inner_size } => {
						window.window.resize(**new_inner_size);
					},
					_ => (),
				}

				window.process_event(&event);
			},
			_ => (),
		}
	});

	std::process::exit(exit_code);
}

struct Window {
	ui: egui_glow::winit::EguiGlow,
	window: glutin::WindowedContext<glutin::PossiblyCurrent>,
	gl: Arc<glow::Context>,
}

impl Window {
	fn new<T>(event_loop: &glutin::event_loop::EventLoopWindowTarget<T>, title: &str) -> Result<Self, ()> {
		let window = glutin::window::WindowBuilder::new()
			.with_x11_window_type(vec![glutin::platform::unix::XWindowType::Dock])
			.with_decorations(false)
			.with_title(title)
			.with_visible(false);

		unsafe {
			let window = glutin::ContextBuilder::new()
				.with_depth_buffer(0)
				.with_stencil_buffer(0)
				.with_srgb(true)
				.build_windowed(window, event_loop)
				.map_err(|e| log::error!("Failed to create window or OpenGL context: {}", e))?
				.make_current()
				.map_err(|_| log::error!("Failed to make OpenGL context current"))?;

			let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s));
			let gl = Arc::new(gl);

			let ui = egui_glow::winit::EguiGlow::new(event_loop, gl.clone());

			Ok(Self {
				ui,
				window,
				gl,
			})
		}
	}

	fn run(&mut self) -> Duration {
		self.ui.run(self.window.window(), |context| {
			context.set_fonts(Default::default());
			egui::SidePanel::left("side-panel").show(context, |ui| {
				ui.heading("Hellow world!");
				if ui.button("Booh!").clicked() {
					println!("Booh!");
				}
			});
		})
	}

	fn native(&self) -> &glutin::window::Window {
		self.window.window()
	}

	fn process_event(&mut self, event: &glutin::event::WindowEvent) {
		self.ui.on_event(event);
		self.window.window().request_redraw()
	}
}
