use glutin::platform::run_return::EventLoopExtRunReturn;
use std::time::{Duration, Instant};

mod bar;
use bar::Bar;

mod logging;
mod state_update;
mod x11;

#[derive(clap::Parser)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
struct Options {
	#[clap(short, long)]
	#[clap(action = clap::ArgAction::Count)]
	verbose: u8,

	#[clap(short, long)]
	#[clap(action = clap::ArgAction::Count)]
	quiet: u8,

	#[clap(short, long)]
	screen: Option<usize>,

	#[clap(short, long, arg_enum)]
	#[clap(default_value_t = Size::Screen)]
	width: Size,

	#[clap(short, long, arg_enum)]
	#[clap(default_value_t = Size::Fit)]
	height: Size,
}

#[derive(Clone)]
pub enum Size {
	Screen,
	Fit,
	Fixed(u32),
}

impl clap::ValueEnum for Size {
	fn from_str(data: &str, ignore_case: bool) -> Result<Self, String> {
		let data = data.trim();
		if compare_str(data, "screen", ignore_case) {
			Ok(Self::Screen)
		} else if compare_str(data, "fit", ignore_case) {
			Ok(Self::Fit)
		} else if let Ok(fixed) = data.parse() {
			Ok(Self::Fixed(fixed))
		} else {
			Err("invalid size".to_owned())
		}
	}

	fn value_variants<'a>() -> &'a [Self] {
		const VARIANTS: &[Size] = &[
			Size::Screen,
			Size::Fit,
			Size::Fixed(0),
		];
		VARIANTS
	}

	fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
		match self {
			Self::Screen => Some(clap::PossibleValue::new("screen").help("Fill the width/height of the screen.")),
			Self::Fit => Some(clap::PossibleValue::new("fit").help("Fit the width/height to the contents.")),
			Self::Fixed(_) => Some(clap::PossibleValue::new("NUMBER").help("Set the width/height to a fixed size in pixels.")),
		}
	}
}

fn compare_str(a: &str, b: &str, ignore_case: bool) -> bool {
	if ignore_case {
		a.eq_ignore_ascii_case(b)
	} else {
		a == b
	}
}

fn main() {
	if let Err(()) = do_main(clap::Parser::parse()) {
		std::process::exit(1);
	}
}

fn do_main(options: Options) -> Result<(), ()> {
	logging::init(module_path!(), &[], options.verbose as i8 - options.quiet as i8);

	let wm_info = x11::WindowManagerInfo::new()?;

	let mut event_loop = glutin::event_loop::EventLoopBuilder::with_user_event().build();
	let mut bar = Bar::new(&event_loop, "mbar", options.screen, options.width, options.height)?;
	bar.window().set_visible(true);

	std::thread::spawn({
		let event_loop = event_loop.create_proxy();
		move || {
			wm_info.run(&event_loop).ok();
		}
	});

	let exit_code = event_loop.run_return(|event, _event_loop, control_flow| {
		match event {
			glutin::event::Event::LoopDestroyed => {
				bar.destroy();
			},
			glutin::event::Event::RedrawRequested(_window_id) => {
				let repaint_after = bar.run();
				if repaint_after.is_zero() {
					bar.window().request_redraw();
				} else {
					let repaint_after = repaint_after.min(Duration::from_millis(50));
					control_flow.set_wait_until(Instant::now() + repaint_after);
				}

				bar.draw();
			},
			glutin::event::Event::NewEvents(glutin::event::StartCause::ResumeTimeReached { .. }) => {
				bar.request_redraw();
			},
			glutin::event::Event::WindowEvent { window_id: _, event } => {

				bar.process_event(&event);
			},
			glutin::event::Event::UserEvent(state_update) => {
				bar.handle_state(&state_update);
			},
			_ => (),
		}
	});

	std::process::exit(exit_code);
}
