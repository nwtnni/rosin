use core::ops::Add;
use core::time::Duration;

use aarch64_cpu::asm;
use aarch64_cpu::registers::CNTFRQ_EL0;
use aarch64_cpu::registers::CNTPCT_EL0;
use tock_registers::interfaces::Readable as _;

use crate::dev::bcm2837b0;

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Instant(Cycle);

impl From<Instant> for Duration {
    fn from(instant: Instant) -> Self {
        Duration::from(instant.0)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;
    fn add(self, delta: Duration) -> Self::Output {
        Self(self.0 + Cycle::from(delta))
    }
}

impl Add<Cycle> for Instant {
    type Output = Self;
    fn add(self, delta: Cycle) -> Self::Output {
        Self(self.0 + delta)
    }
}

impl Instant {
    pub fn now() -> Self {
        asm::barrier::isb(asm::barrier::SY);
        Instant(Cycle(CNTPCT_EL0.get()))
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Cycle(u64);

impl Cycle {
    pub const ONE: Cycle = Cycle(1);

    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    // TOOD: hide
    pub fn value(self) -> u64 {
        self.0
    }
}

impl From<Duration> for Cycle {
    fn from(duration: Duration) -> Self {
        let frequency = frequency();
        let s = duration.as_secs() * frequency;
        let ns = duration.subsec_nanos() as u64 * frequency / 10u64.pow(9);
        Self(s + ns)
    }
}

impl From<Cycle> for Duration {
    fn from(Cycle(cycle): Cycle) -> Self {
        let frequency = frequency();
        let s = cycle / frequency;
        let ns = (cycle % frequency) * 10u64.pow(9) / frequency;
        Duration::new(s, ns as u32)
    }
}

impl Add for Cycle {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

pub fn spin(duration: Duration) {
    let start = Instant::now();
    let stop = start + duration;
    while CNTPCT_EL0.get() < stop.0.0 {}
}

pub fn spin_cycle(duration: Cycle) {
    let start = Instant::now();
    let stop = start + duration;
    while CNTPCT_EL0.get() < stop.0.0 {}
}

pub fn frequency() -> u64 {
    CNTFRQ_EL0.get()
}
