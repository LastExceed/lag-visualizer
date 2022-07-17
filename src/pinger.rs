use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use ekko::EkkoResponse;
use crate::capturer::PEER_IP;
use crate::display::Model;

pub struct Ping {
	pub timestamp: Instant,
	pub duration: Option<Duration>
}

pub fn begin_loop_ping(model_arc: Arc<Mutex<Model>>) {
	let ekko = ekko::Ekko::with_target(PEER_IP).unwrap();
	loop {
		{//todo: ugly
			let now = Instant::now();
			let mut model = model_arc.lock().unwrap();
			model.pings.push_front(Ping {
				timestamp: now,
				duration: None
			});

			loop {
				if let Some(record) = model.pings.back() {
					if now - record.timestamp > Duration::from_secs(10) {
						model.pings.pop_back();
						continue;
					}
				}
				break;
			}
		}
		if let EkkoResponse::Destination(ekko_response) = ekko.send(128).unwrap() {
			model_arc.lock().unwrap().pings[0].duration = Some(ekko_response.elapsed);
		};
		thread::sleep(Duration::from_millis(200));
	}
}