use glutin::platform::unix::WindowBuilderExtUnix;
use std::sync::Arc;
use std::time::Duration;

use crate::Size;


pub struct Bar {
	ui: egui_glow::winit::EguiGlow,
	window: glutin::WindowedContext<glutin::PossiblyCurrent>,
	gl: Arc<glow::Context>,
	width: Option<u32>,
	height: Option<u32>,
	state: State,
}

struct State {
	content_size: Option<egui::Vec2>,
}

impl Bar {
	pub fn new<T>(
		event_loop: &glutin::event_loop::EventLoopWindowTarget<T>,
		title: &str,
		screen: Option<usize>,
		width: Size,
		height: Size,
	) -> Result<Self, ()> {
		let monitor = match screen {
			None => event_loop.primary_monitor().ok_or_else(|| log::error!("Failed to determine primary monitor."))?,
			Some(i) => event_loop.available_monitors().nth(i).ok_or_else(|| {
				let monitor_count = event_loop.available_monitors().count();
				log::error!("Invalid screen index, got {}, but there are only {} screens.", i, monitor_count);
			})?
		};

		let width = match width {
			Size::Screen => Some(monitor.size().width),
			Size::Fit => None,
			Size::Fixed(x) => Some(x),
		};

		let height = match height {
			Size::Screen => Some(monitor.size().height),
			Size::Fit => None,
			Size::Fixed(x) => Some(x),
		};

		let window_size = glutin::dpi::PhysicalSize::new(
			width.unwrap_or_else(|| monitor.size().width),
			height.unwrap_or_else(|| monitor.size().height),
		);

		let window = glutin::window::WindowBuilder::new()
			.with_x11_window_type(vec![glutin::platform::unix::XWindowType::Dock])
			.with_decorations(false)
			.with_title(title)
			.with_inner_size(window_size)
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
				state: State::new(),
				width,
				height,
			})
		}
	}

	pub fn window(&self) -> &glutin::window::Window {
		self.window.window()
	}

	pub fn destroy(&mut self) {
		self.ui.destroy();
	}

	pub fn run(&mut self) -> Duration {
		let time_to_redraw = self.ui.run(self.window.window(), |context| {
			self.state.draw(context);
		});

		if let Some(content_size) = self.state.content_size {
			let content_size = glutin::dpi::PhysicalSize::new(
				self.width.unwrap_or(content_size.x.round() as u32),
				self.height.unwrap_or(content_size.y.round() as u32),
			);

			if self.window().inner_size() != content_size {
				log::debug!("Resizing window to {}x{}", content_size.width, content_size.height);
				self.window().set_inner_size(content_size);
			}
		}

		time_to_redraw
	}

	pub fn request_redraw(&self) {
		self.window().request_redraw()
	}

	pub fn draw(&mut self) {
		unsafe {
			use glow::HasContext;
			self.gl.clear_color(1.0, 0.0, 1.0, 1.0);
			self.gl.clear(glow::COLOR_BUFFER_BIT);
		}

		self.ui.paint(self.window.window());
		if let Err(e) = self.window.swap_buffers() {
			log::error!("Failed to swap window buffers: {}", e);
		}
	}

	pub fn process_event(&mut self, event: &glutin::event::WindowEvent) {
		match &event {
			glutin::event::WindowEvent::CloseRequested | glutin::event::WindowEvent::Destroyed => {
				self.destroy();
			},
			glutin::event::WindowEvent::Resized(size) => {
				log::trace!("Resizing EguiGlow in response to Resized event: {}x{}", size.width, size.height);
				self.window.resize(*size);
			},
			glutin::event::WindowEvent::ScaleFactorChanged { scale_factor: _, new_inner_size } => {
				let size = **new_inner_size;
				log::trace!("Resizing EguiGlow in response to ScaleFactorChanged event: {}x{}", size.width, size.height);
				self.window.resize(size);
			},
			_ => (),
		}
		self.ui.on_event(event);
		self.window.window().request_redraw()
	}
}

impl State {
	fn new() -> Self {
		Self {
			content_size: None,
		}
	}

	fn draw(&mut self, context: &egui::Context) {
		let mut panel_ui_size = Default::default();
		let mut content_size = Default::default();
		egui::CentralPanel::default().show(context, |ui| {
			log::trace!("available size for panel ui: {:?}", ui.available_size());
			ui.horizontal_top(|ui| {
				ui.label("workspaces");
				ui.add_space(10.0);
				ui.label("Layout");
				ui.add_space(10.0);
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Min).with_cross_justify(false), |ui| {
					self.draw_clock(ui);
					ui.add_space(10.0);
					ui.shrink_height_to_current();
					ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight).with_cross_justify(false), |ui| {
						ui.label("Title of focussed window");
						log::trace!("center ui size: {:?}", ui.min_size());
					});
					log::trace!("right ui size: {:?}", ui.min_size());
				});
				log::trace!("left ui size: {:?}", ui.min_size());
				content_size = ui.min_size();
			});
			log::trace!("panel ui size: {:?}", ui.min_size());
			panel_ui_size = ui.min_size();
		});
		log::trace!("context used size: {:?}", context.used_rect().size());
		self.content_size = Some(context.used_size() - panel_ui_size + content_size);
	}

	fn draw_clock(&self, ui: &mut egui::Ui) {
		let now = chrono::Local::now();
		let time = now.format("%a %b %d %Y %H:%M:%S");
		ui.label(time.to_string());
	}
}

