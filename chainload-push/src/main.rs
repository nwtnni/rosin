use core::fmt::Display;
use core::time::Duration;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read as _;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser as _;
use serialport::ClearBuffer;
use serialport::DataBits;
use serialport::FlowControl;
use serialport::Parity;
use serialport::StopBits;

#[derive(clap::Parser)]
struct Cli {
    #[arg(short, long, default_value_t = 115_200)]
    baud: u32,

    #[arg(short, long, default_value = "/dev/ttyUSB0")]
    port: String,

    #[arg(short, long, default_value = "kernel8.img")]
    kernel: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let file = File::open(&cli.kernel).unwrap();
    let meta = file.metadata().unwrap();
    let mut file = BufReader::new(file);

    let mut port = serialport::new(&cli.port, cli.baud)
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
    let eta = Duration::from_secs(len as u64 * 8 / (cli.baud as u64));

    eprintln!(
        "[PUSH] Sending {} at {} baud (~{})",
        Memory(len),
        cli.baud,
        Time(eta),
    );

    port.write_all(&len.to_le_bytes()).unwrap();

    const CHUNK: usize = 512;

    let chunks = len.next_multiple_of(CHUNK) / CHUNK;

    let start = Instant::now();
    for i in 0..chunks {
        let now = start.elapsed();

        eprint!(
            "\r[PUSH] {} / {} | {} / {} | {:.01}",
            Memory(i * CHUNK),
            Memory(len),
            Time(now),
            Time(eta),
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

struct Memory(usize);

impl Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shift = (0..40)
            .step_by(10)
            .find(|shift| self.0 < (1 << (shift + 10)))
            .unwrap_or(40);

        let unit = match shift {
            0 => "B",
            10 => "KiB",
            20 => "MiB",
            30 => "GiB",
            _ => "TiB",
        };

        (self.0 >> shift).fmt(f)?;
        unit.fmt(f)
    }
}

struct Time(Duration);

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let min = self.0.as_secs() / 60;
        let sec = self.0.as_secs() % 60;
        write!(f, "{:02}:{:02}", min, sec)
    }
}
