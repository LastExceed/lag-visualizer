use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use packet::{ether, ip, Packet, tcp};
use crate::display::Model;

pub const XIV_IP: [u8; 4] = [195, 82, 50, 46];

pub struct OutRecord {
	pub timestamp: Instant,
	pub duration: Option<Duration>,
	acknowledging_sequence: u32,
}

pub fn begin_loop_capture(model_arc: Arc<Mutex<Model>>) {
	let mut device_list = pcap::Device::list().unwrap();
	let device = device_list.remove(3);

	let mut cap = pcap::Capture::from_device(device)
		.unwrap()
		.immediate_mode(true)
		.open()
		.unwrap();

	loop {
		let raw_packet = match cap.next() {
			Ok(packet) => packet,
			Err(_) => {
				continue;
			}
		};
		let now = Instant::now();
		let mut model = model_arc.lock().unwrap();

		let ether_packet = ether::Packet::new(raw_packet.data).unwrap();
		if ether_packet.protocol() != ether::Protocol::Ipv4 { continue; }

		let ip_packet = ip::v4::Packet::new(ether_packet.payload()).unwrap();
		if ip_packet.protocol() != ip::Protocol::Tcp { continue; }

		let tcp_packet = if let Ok(tcp_packet) = tcp::Packet::new(ip_packet.payload()) {
			tcp_packet
		} else { continue; };

		if ip_packet.source().octets() == XIV_IP {
			let ack = tcp_packet.acknowledgment();

			for i in 0..model.sent.len() {//todo
				let entry = &mut model.sent[i];
				if entry.acknowledging_sequence == ack {
					if entry.duration == None {
						entry.duration = Some(now - entry.timestamp);
					}
					break;
				}
			}
			model.received.push_front(now);

			loop {
				if let Some(record) = model.received.back() {
					if now - *record > Duration::from_secs(10) {
						model.received.pop_back();
						continue;
					}
				}
				break;
			}
		} else if ip_packet.destination().octets() == XIV_IP {
			if !tcp_packet.flags().contains(tcp::flag::PSH) { continue; }
			let new_entry = OutRecord {
				timestamp: now,
				duration: None,
				acknowledging_sequence: tcp_packet.sequence() + (tcp_packet.payload().len() as u32)
			};
			model.sent.push_front(new_entry);

			loop {
				if let Some(record) = model.sent.back() {
					if now - record.timestamp > Duration::from_secs(10) {
						model.sent.pop_back();
						continue;
					}
				}
				break;
			}
		}
	}
}