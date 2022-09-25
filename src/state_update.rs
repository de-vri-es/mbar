#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StateUpdate {
	pub desktop_names: Vec<String>,
	pub desktop_layout: String,
}
