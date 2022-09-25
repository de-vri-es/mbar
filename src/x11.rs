use std::time::Duration;
use xcb_wm::ewmh;

use crate::state_update::StateUpdate;

pub struct WindowManagerInfo {
	x11: xcb::Connection,
	last_state: StateUpdate,
}

impl WindowManagerInfo {
	pub fn new() -> Result<Self, ()> {
		let (x11, _screen) = xcb::Connection::connect(None)
			.map_err(|e| log::error!("Failed to connect to X11 server: {}", e))?;
		Ok(Self {
			x11,
			last_state: StateUpdate {
				desktop_names: Vec::new(),
				desktop_layout: String::new(),
			},
		})
	}

	pub fn desktop_names(&self) -> Result<Vec<String>, ()> {
		let ewhm = ewmh::Connection::connect(&self.x11);
		let cookie = ewhm.send_request(&ewmh::proto::GetDesktopNames);
		let reply = ewhm.wait_for_reply(cookie)
			.map_err(|e| log::error!("Failed to get desktop names: {}", e))?;
		Ok(reply.names)
	}

	pub fn current_desktop(&self) -> Result<u32, ()> {
		let ewhm = ewmh::Connection::connect(&self.x11);
		let cookie = ewhm.send_request(&ewmh::proto::GetCurrentDesktop);
		let reply = ewhm.wait_for_reply(cookie)
			.map_err(|e| log::error!("Failed to get desktop names: {}", e))?;
		Ok(reply.desktop)
	}

	pub fn active_window(&self) -> Result<xcb::x::Window, ()> {
		let ewhm = ewmh::Connection::connect(&self.x11);
		let cookie = ewhm.send_request(&ewmh::proto::GetActiveWindow);
		let reply = ewhm.wait_for_reply(cookie)
			.map_err(|e| log::error!("Failed to get desktop names: {}", e))?;
		Ok(reply.window)
	}

	pub fn run(&self, event_loop_proxy: &glutin::event_loop::EventLoopProxy<StateUpdate>) -> Result<(), ()> {
		loop {
			let new_state = StateUpdate {
				desktop_names: self.desktop_names()?,
				desktop_layout: "".into(),
			};
			match event_loop_proxy.send_event(new_state) {
				Ok(()) => std::thread::sleep(Duration::from_millis(50)),
				Err(glutin::event_loop::EventLoopClosed(_update)) => return Ok(()),
			};
		}
	}
}
