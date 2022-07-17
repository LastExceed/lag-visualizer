use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use nannou::prelude::*;
use crate::{capturer, pinger};
use crate::capturer::OutRecord;
use crate::pinger::Ping;

pub struct Model {
	pub sent: VecDeque<OutRecord>,
	pub received: VecDeque<Instant>,
	pub pings: VecDeque<Ping>
}

pub fn create_model(_app: &App) -> Arc<Mutex<Model>> {
	_app.main_window().winit_window().set_title("lag-o-meter");

	let model = Model {
		sent: Default::default(),
		received: Default::default(),
		pings: Default::default()
	};

	let model_arc1 = Arc::new(Mutex::new(model));
	let model_arc2 = Arc::clone(&model_arc1);
	let model_arc3 = Arc::clone(&model_arc1);

	thread::spawn(move || {
		capturer::begin_loop_capture(model_arc2);
	});
	thread::spawn(move || {
		pinger::begin_loop_ping(model_arc3);
	});

	model_arc1
}

pub fn view(app: &App, _model: &Arc<Mutex<Model>>, frame: Frame) {
	frame.clear(BLACK);
	let draw = app.draw();
	let rect = frame.rect();

	let model = _model.lock().unwrap();
	let now = Instant::now();

	let mut i = 1;
	loop {
		let y_delta = i as f32 * 50f32;
		let y = rect.y.start + y_delta;
		if y > rect.y.end { break; }

		draw.line()
			.color(GRAY)
			.start(pt2(rect.x.start, y))
			.end(pt2(rect.x.end, y))
			.finish();

		for x in [rect.x.start + 10f32, rect.x.end - 10f32] {
			draw.text(y_delta.to_string().as_str())
				.color(GRAY)
				.x_y(x, y + 10f32)
				.finish();
		}

		i += 1;
	}

	draw.text("received packet timeout")
		.color(CYAN)
		.y(rect.y.end - 10f32)
		.finish();

	draw.text("time to acknowledge")
		.color(YELLOW)
		.y(rect.y.end - 30f32)
		.finish();

	draw.text("ICMP ping")
		.color(MAGENTA)
		.y(rect.y.end - 50f32)
		.finish();

	let draw_record = |
		color: Srgb<u8>,
		timestamp: Instant,
		height: Duration
	| {
		let x = rect.x.end - (now - timestamp).as_millis() as f32 / 3f32;

		if x < rect.x.start { return; };

		draw.line()
			.color(color)
			.start(pt2(x, rect.y.start))
			.end(pt2(x, rect.y.start + height.as_millis() as f32))
			.finish();
	};

	for sent_packet in model.sent.iter() {
		draw_record(
			YELLOW,
			sent_packet.timestamp,
			sent_packet.duration.unwrap_or(now - sent_packet.timestamp)
		);

	}

	let mut previous: Option<Instant> = None;
	for entry in model.received.iter() {
		draw_record(
			CYAN,
			*entry,
			previous.unwrap_or(now) - *entry
		);
		previous = Some(*entry);
	}

	for ping in model.pings.iter() {
		draw_record(
			MAGENTA,
			ping.timestamp,
			ping.duration.unwrap_or(now - ping.timestamp)
		);
	}

	draw.to_frame(app, &frame).unwrap();
}