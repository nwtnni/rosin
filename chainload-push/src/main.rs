use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read as _;

use serialport::ClearBuffer;
use serialport::DataBits;
use serialport::FlowControl;
use serialport::Parity;
use serialport::StopBits;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let file = File::open(path).unwrap();
    let meta = file.metadata().unwrap();
    let mut file = BufReader::new(file);

    let mut port = serialport::new("/dev/ttyUSB0", 115_200)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .timeout(std::time::Duration::MAX)
        .open()
        .expect("Failed to open port");

    port.clear(ClearBuffer::All).unwrap();

    eprintln!("[PUSH] Waiting for handshake...");

    let mut buffer = [0u8; 1];
    let mut len = 0;

    while len < 8 {
        match port.read(&mut buffer) {
            Ok(1) if buffer[0] == 0xff => len += 1,
            Ok(_) => (),
            Err(error) if matches!(error.kind(), io::ErrorKind::TimedOut) => (),
            error => error.map(drop).unwrap(),
        }
    }

    port.write_all(&[0xff; 8]).unwrap();

    let len = meta.len() as usize;

    eprintln!("[PUSH] Sending {} bytes...", len);

    port.write_all(&len.to_le_bytes()).unwrap();

    const CHUNK: usize = 512;

    let chunks = len.next_multiple_of(CHUNK) / CHUNK;

    for i in 0..chunks {
        eprint!(
            "\r[PUSH] {} / {} ({:.01}%)",
            i * CHUNK,
            len,
            ((i * CHUNK * 100) as f64) / (len as f64),
        );
        std::io::copy(&mut file.by_ref().take(CHUNK as u64), &mut port).unwrap();

        if i == chunks - 1 {
            eprintln!();
        }
    }

    let mut stdout = std::io::stdout().lock();
    std::io::copy(&mut port, &mut stdout).unwrap();
}
