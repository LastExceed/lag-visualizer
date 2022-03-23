use crate::display::{create_model, view};

mod capturer;
mod display;
mod pinger;

fn main() {
	nannou::app(create_model).simple_window(view).run();
}