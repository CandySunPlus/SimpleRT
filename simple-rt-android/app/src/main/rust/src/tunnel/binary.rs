use std::cmp::min;
use std::fmt::Write;

const MAX_STING_PACKET_SIZE: usize = 24;

#[allow(unused_variables)]
pub fn build_packet_string(data: &[u8]) -> String {
    let mut s = String::new();

    let limit = min(MAX_STING_PACKET_SIZE, data.len());

    for (i, &byte) in data.iter().take(limit).enumerate() {
        if i != 0 {
            let sep = if (i % 4) == 0 { "  " } else { " " };
            write!(&mut s, "{}", sep).unwrap();
        }
        write!(&mut s, "{:02X}", byte).unwrap();
    }
    if limit < data.len() {
        write!(&mut s, " ... +{} bytes", data.len() - limit).unwrap();
    }
    s
}
