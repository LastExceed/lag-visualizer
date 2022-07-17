use crate::display::{create_model, view};

mod capturer;
mod display;
mod pinger;

fn main() {
	let mut device_list = pcap::Device::list().unwrap();
	for device in device_list {
		println!("{}", device.desc.unwrap());
	}
	nannou::app(create_model).simple_window(view).run();
}