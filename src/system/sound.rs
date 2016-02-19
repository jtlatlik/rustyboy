use super::system::MemoryAccess;
use super::ioregister::IORegister;

macro_rules! bits {
	( $($bit:expr)* ) => ( 0x00 $( | (1<<$bit) )* )
}

pub struct WaveRAM([u8; 16]);

pub struct SoundData {
	pub regs : SoundRegisters,
	pub wave_ram : WaveRAM
}

pub struct SoundRegisters {
	//control registers
	pub ctrl_vol : IORegister,
	pub ctrl_ch_mux : IORegister,
	pub ctrl_on_off : IORegister,
	
	//channel registers
	pub ch1_sweep : IORegister,
	pub ch1_length_duty : IORegister,
	pub ch1_vol_env : IORegister,
	pub ch1_freq_low : IORegister,
	pub ch1_freq_high : IORegister,
	pub ch2_length_duty : IORegister,
	pub ch2_vol_env : IORegister,
	pub ch2_freq_low : IORegister,
	pub ch2_freq_high : IORegister,
	pub ch3_snd_on_off : IORegister,
	pub ch3_snd_length : IORegister,
	pub ch3_out_level : IORegister,
	pub ch3_freq_low : IORegister,
	pub ch3_freq_high : IORegister,
	pub ch4_snd_length : IORegister,
	pub ch4_vol_env : IORegister,
	pub ch4_poly_cnt : IORegister,
	pub ch4_cnt_init : IORegister
}

impl Default for SoundData {
	fn default() -> SoundData {
		SoundData {
			regs : SoundRegisters { ..Default::default() },
			wave_ram : WaveRAM([0;16])
		}
	}
}

impl Default for SoundRegisters {
	
	fn default() -> SoundRegisters {
		SoundRegisters {
			ctrl_on_off : IORegister::new().write_mask(bits!(7)),
			ctrl_ch_mux : IORegister::new(),
			ctrl_vol : IORegister::new(),
			ch1_sweep : IORegister::new().write_mask(bits!(6 5 4 3 2 1 0)),
			ch1_length_duty : IORegister::new().read_mask(bits!(7 6)),
			ch1_vol_env : IORegister::new(),
			ch1_freq_low : IORegister::new().write_only(),
			ch1_freq_high : IORegister::new().write_mask(bits!(7 6 2 1 0)).read_mask(bits!(6)),
			ch2_length_duty : IORegister::new().read_mask(bits!(7 6)),
			ch2_vol_env : IORegister::new(),
			ch2_freq_low : IORegister::new().write_only(),
			ch2_freq_high : IORegister::new().write_mask(bits!(7 6 2 1 0)).read_mask(bits!(6)),
			ch3_snd_on_off : IORegister::new().write_mask(bits!(7)),
			ch3_snd_length : IORegister::new(),
			ch3_out_level : IORegister::new().write_mask(bits!(6 5)),
			ch3_freq_low : IORegister::new().write_only(),
			ch3_freq_high : IORegister::new().write_mask(bits!(7 6 2 1 0)).read_mask(bits!(6)),
			ch4_snd_length : IORegister::new().write_mask(bits!(5 4 3 2 1 0)),
			ch4_vol_env : IORegister::new(),
			ch4_poly_cnt : IORegister::new(),
			ch4_cnt_init : IORegister::new().write_mask(bits!(7 6)).read_mask(bits!(6))
		}
	}
}

impl MemoryAccess for WaveRAM {
	
	#[inline(always)]
	fn read(&mut self, addr: u16) -> u8 {
		self.0[addr as usize]
	}
	
	#[inline(always)]
	fn write(&mut self, addr: u16, data: u8) {
		self.0[addr as usize] = data;
	}
}