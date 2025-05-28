use core::ops::Add;
use core::time::Duration;

use aarch64_cpu::asm;
use aarch64_cpu::registers::CNTFRQ_EL0;
use aarch64_cpu::registers::CNTPCT_EL0;
use tock_registers::interfaces::Readable as _;

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Instant(u64);

impl From<Instant> for Duration {
    fn from(Instant(count): Instant) -> Self {
        let frequency = frequency();
        let s = count / frequency;
        let ns = (count % frequency) * 10u64.pow(9) / frequency;
        Duration::new(s, ns as u32)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self::Output {
        let frequency = frequency();

        let s = rhs.as_secs() * frequency;
        let ns = (rhs.as_nanos() * frequency as u128) / 10u128.pow(9);

        Self(self.0 + s + ns as u64)
    }
}

impl Instant {
    pub fn now() -> Self {
        asm::barrier::isb(asm::barrier::SY);
        Instant(CNTPCT_EL0.get())
    }
}

pub fn resolution() -> Duration {
    Duration::from(Instant(1))
}

pub fn spin(duration: Duration) {
    let start = Instant::now();
    let stop = start + duration;
    while CNTPCT_EL0.get() < stop.0 {}
}

fn frequency() -> u64 {
    CNTFRQ_EL0.get()
}
