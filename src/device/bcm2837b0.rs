pub mod gpio;
pub mod uart;

core::arch::global_asm! {
r#"
.pushsection .dtb
.incbin "src/device/bcm2837b0/bcm2710-rpi-3-b-plus.dtb"
.popsection
"#
}
