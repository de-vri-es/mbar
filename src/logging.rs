use std::io::Write;

/// Initialize the logging system with a pretty format.
///
/// Logging for the specified root module will be set to Error, Warn, Info, Debug or Trace, depending on the verbosity parameter.
/// Logging for all other modules is set to one level less verbose.
pub(crate) fn init(root_module: &str, extra_modules: &[&str], verbosity: i8) {
	let log_level = match verbosity {
		i8::MIN..=-2 => log::LevelFilter::Error,
		-1 => log::LevelFilter::Warn,
		0 => log::LevelFilter::Info,
		1 => log::LevelFilter::Debug,
		2..=i8::MAX => log::LevelFilter::Trace,
	};

	let extra_level = match verbosity {
		i8::MIN..=-1 => log::LevelFilter::Error,
		0 => log::LevelFilter::Warn,
		1 => log::LevelFilter::Info,
		2 => log::LevelFilter::Debug,
		3..=i8::MAX => log::LevelFilter::Trace,
	};

	let mut logger = env_logger::Builder::new();
	logger
		.format(move |formatter, record| format_msg(formatter, record))
		.filter_level(log::LevelFilter::Warn)
		.filter_module(root_module, log_level);

	for module in extra_modules {
		logger.filter_module(module, extra_level);
	}

	logger.init();
}

fn format_msg(formatter: &mut env_logger::fmt::Formatter, record: &log::Record) -> std::io::Result<()> {
	use env_logger::fmt::Color;

	let now = chrono::Local::now();

	let mut prefix_style = formatter.style();
	let prefix;

	match record.level() {
		log::Level::Trace => {
			prefix = "Trace:";
			prefix_style.set_bold(true);
		},
		log::Level::Debug => {
			prefix = "Debug:";
			prefix_style.set_bold(true);
		},
		log::Level::Info => {
			prefix = "Info:";
			prefix_style.set_bold(true);
		},
		log::Level::Warn => {
			prefix = "Warn:";
			prefix_style.set_color(Color::Yellow).set_bold(true);
		},
		log::Level::Error => {
			prefix = "Error:";
			prefix_style.set_color(Color::Red).set_bold(true);
		},
	};

	write!(
		formatter,
		"[{}] ",
		formatter.style().set_color(Color::Cyan).value(now.format("%Y-%m-%d %H:%M:%S%.6f %z")),
	)?;

	write!(
		formatter,
		"{} ",
		prefix_style.value(prefix),
	)?;

	writeln!(formatter, "{}", record.args())
}
