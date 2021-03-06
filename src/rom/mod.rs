pub mod header;

use std::fs::File;
use std::io::Read;
use std::io;
use std::str;
use std::mem;


use self::header::*;
use self::header::RomSize::*;
use self::header::CartridgeType::*;

pub const NUM_ROM_BANK_BYTES : usize = 16384;
pub type RomBank = Box<[u8; NUM_ROM_BANK_BYTES]>;

pub struct Rom {
	pub filename : String,
    pub banks : Vec<RomBank>,
    pub title : String,
    pub cgb_flag : bool,
    pub sgb_flag : bool,
    pub rom_type : CartridgeType,
    pub rom_size : RomSize,
    pub rom_manufacturer : [u8; 4],
    pub ram_size : RamSize,
    battery : bool
}

impl Rom {
    
    fn get_size_in_bytes(rom_size : &RomSize) -> usize {
        match *rom_size {
            ROM_1M1 => 1153434,
            ROM_1M2 => 1258292,
            ROM_1M5 => 1572864,
            ref s @ _ => {
                32768 << *s as u64
            }
        }
    }

    pub fn has_battery(&self) -> bool {
    	self.battery
    }

    pub fn create_from_file(filename : &str) -> Result<Rom, io::Error> {
    	
    	info!("Opening ROM {}", filename);
        //Open file for reading...
        let mut file = try!(File::open(filename));
        
        let mut banks = Vec::new();
        
        
        info!("Reading ROM header...");
        
        //read first bank
        let mut data = [0; NUM_ROM_BANK_BYTES];
        try!(file.read_exact(&mut data));
        banks.push(Box::new(data));
        
        let rom_type : CartridgeType = unsafe { mem::transmute(data[0x147]) };
        let rom_size : RomSize =  unsafe { mem::transmute(data[0x148]) };
        let ram_size : RamSize = unsafe { mem::transmute(data[0x149]) };
        
        let mut title_bytes : [u8; 16] = [0; 16];
        for i in 0..16 {
            title_bytes[i] = data[0x134 + i];
        }
        let title = match str::from_utf8(&title_bytes) {
            Ok(s) => s,
            _ => "UNKNOWN"
        };
        let rom_manufacturer : [u8; 4] = [title_bytes[11], title_bytes[12], title_bytes[13], title_bytes[14]];
        
        //now we know the size of the ROM. read remaining banks.
        let mut num_remaining_bytes = Rom::get_size_in_bytes(&rom_size) - NUM_ROM_BANK_BYTES;
        assert!(num_remaining_bytes % NUM_ROM_BANK_BYTES == 0);
        info!("Reading {} more bytes...", num_remaining_bytes);        
        while num_remaining_bytes > 0 {
            let mut data = [0; NUM_ROM_BANK_BYTES];
            try!(file.read_exact(&mut data));
            banks.push(Box::new(data));
            num_remaining_bytes-=NUM_ROM_BANK_BYTES;
        }
        
        let battery = match rom_type {
        	MBC1_RAM_BATTERY | MBC2_BATTERY | ROM_RAM_BATTERY | MMM01_RAM_BATTERY |
        	MBC3_TIMER_RAM_BATTERY | MBC3_RAM_BATTERY | MBC4_RAM_BATTERY | MBC5_RAM_BATTERY |
        	MBC5_RUMBLE_RAM_BATTERY | HUC1_RAM_BATTERY => true,
        	_ => false
        };
        
		info!("Successfully read ROM");
        Ok(Rom {
        		filename : filename.to_string(),
        		banks : banks,
        		title : title.to_string(),
        		cgb_flag : true,
        		sgb_flag : true,
        		rom_type : rom_type,
        		rom_size : rom_size,
        		rom_manufacturer : rom_manufacturer,
        		battery : battery,
        		ram_size : ram_size
          })
    }
    
    pub fn dump_header(&self) {
        println!("Title: {}", self.title);
        println!("ROM Type: {:?}", self.rom_type);
        println!("ROM Size: {:?}", self.rom_size);
        println!("ROM Manufacturer: {:?}", self.rom_manufacturer);
    }
}