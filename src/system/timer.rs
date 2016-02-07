#[derive(Default)]
pub struct TimerRegisters {
	pub divider : u8, //TODO: unimplemented behavior: writing do div resets contents.
	pub counter : u8,
	pub modulo : u8,
	pub control : u8
}