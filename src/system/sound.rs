#[derive(Default)]
pub struct SoundRegisters {
	//control registers
	pub ctrl_vol : u8,
	pub ctrl_ch_mux : u8,
	pub ctrl_on_off : u8,
	
	//channel registers
	pub ch1_sweep : u8,
	pub ch1_length_duty : u8,
	pub ch1_vol_env : u8,
	pub ch1_freq_low : u8,
	pub ch1_freq_high : u8,
	pub ch2_length_duty : u8,
	pub ch2_vol_env : u8,
	pub ch2_freq_low : u8,
	pub ch2_freq_high : u8,
	pub ch3_snd_on_off : u8,
	pub ch3_snd_length : u8,
	pub ch3_out_level : u8,
	pub ch3_freq_low : u8,
	pub ch3_freq_high : u8,
	pub ch4_snd_length : u8,
	pub ch4_vol_env : u8,
	pub ch4_poly_cnt : u8,
	pub ch4_cnt_init : u8
}

