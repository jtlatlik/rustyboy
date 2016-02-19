use std::thread;
use std::collections::HashMap;
use super::sdl2::pixels::PixelFormatEnum;
use super::sdl2::pixels::Color;
use super::sdl2::rect::Point;
use super::sdl2::rect::Rect;
use super::sdl2::event::Event;
use super::sdl2::keyboard::Keycode;
use super::sdl2::render::*;

use std::process;

use system::system::GBSystem;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Receiver;

use system::video::*;

static mut renderer_inst : Option<Renderer> = None; 

const SCALE : u32 = 4;
const TILE_SIZE :u32 = SCALE*8;
const TILE_OFFSET :u32 = TILE_SIZE+1; 

const WIDTH : u32 = TILE_OFFSET*16 - 1;
const HEIGHT : u32 = TILE_OFFSET*16 - 1;
const NUM_SPRITES : usize = 40;
const NUM_SPRITE_BYTES : usize = 4;

struct Tileset {
	texture: Texture
}

type Palette = [Color; 4];

pub fn display(vregs : Arc<RwLock<VideoRegisters>>, vram : Arc<RwLock<VRAMBank>>, v_rx : Receiver<VideoEvent>, oam : Arc<RwLock<OAM>>) {

    let sdl_context = super::sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("Tile Viewer", WIDTH*2, HEIGHT)
    	.position_centered()
    	.opengl()
    	.build()
    	.unwrap();

	renderer_inst = Some(window.renderer().target_texture().build().unwrap());
	//let mut event_pump = sdl_context.event_pump().unwrap();
	
	//let mut bg = renderer.create_texture_target(PixelFormatEnum::RGB24, (256,256));
//	let tileset = Tileset::new(&mut renderer);

	let mut bg_palette = [Color::RGB(0,0,0); 4];

	renderer.set_draw_color(Color::RGB(0,0,0));
	rendererenderer.clear();
}

pub fn update() {
	
	
	let mut renderer = renderer_inst.unwrap();
    'running: loop {
    	
    	match v_rx.recv().unwrap() {
    		VideoEvent::SCANLINE_OAM => continue, //TODO read OAM data only during this mode
    		VideoEvent::SCANLINE_VRAM =>(), 
    		_ => continue
    	}

//        for event in event_pump.poll_iter() {
//            match event {
//                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
//                	process::exit(0);
//                    //break 'running
//                },
//                _ => {}
//            }
//        }


    }
}

/*
impl Tileset {
	
	pub fn new(renderer : &mut Renderer) -> Tileset {
		
		let texture = renderer.create_texture_streaming( PixelFormatEnum::RGB24, (128,128)).unwrap();
		
		let mut tileset = Tileset { 
			texture : texture
		};

		tileset
	}
	
	pub fn update(&mut self, vram : &VRAMBank, vregs : &VideoRegisters) {

		let vram_tileset_addr = if ((*vregs.lcd_ctrl >> LCD_CONTROL_TSSEL) & 1) != 0 { 0 } else { 0x800 };
		let vram = &vram.data;
		let mut palette = [Color::RGB(0,0,0); 4];
		for i in 0..4 {
			let c = (*vregs.bgp >> (2*i)) & 0x3;
			palette[i] = match c {
				0 => Color::RGB(255,255,255),
				1 => Color::RGB(192,192,192),
				2 => Color::RGB(96,96,96),
				3 => Color::RGB(0,0,0),
				_ => unreachable!()
			}
		}
		
		self.texture.with_lock(None, |buffer: &mut [u8], pitch :usize| {
			for t  in 0..256 {
				let tile_addr = vram_tileset_addr as usize + t*16;

				let texture_tile_offset = (t/16)*pitch*8 + (t%16)*(pitch/16);
				
				for y in 0..8 {
					let tile_data = [ vram[tile_addr + y*2],  vram[tile_addr + y*2 + 1]];
					for x in 0..8 {
						let index = (((tile_data[1] >> (7-x))&1) << 1) | ((tile_data[0] >> (7-x))&1);
						if let Color::RGB(r,g,b) = palette[index as usize] {
							let offset = texture_tile_offset + y*pitch + x*3;
							buffer[offset +0] = r;
							buffer[offset +1] = g;
							buffer[offset +2] = b;
						}
						
					}
				}
	
			}
		}).unwrap();
		
	} 
}
*/
//impl BG {
//	
//	pub fn render()
//}

//struct BG(Texture, Palette);
//
//struct Sprite {
//	pos : Point,
//	tile : u8,
//	xflip : bool,
//	yflip : bool,
//	priority : bool,
//	palette : u8
//}
//
//fn read_sprites(oam : &Arc<RwLock<OAM>>) -> Vec<Sprite> {
//	let oam = &oam.read().unwrap().data;
//	let mut sprites : Vec<Sprite> = Vec::new();
//
//	for i in 0..NUM_SPRITES {
//		 let addr = i*NUM_SPRITE_BYTES;
//		 let attrs = oam[addr+3];
//		 let s = Sprite {
//		 	pos : Point::new(oam[addr+1] as i32 + 8, oam[addr] as i32 + 16),
//		 	tile : oam[addr+2],
//		 	priority : attrs&(1<<7) != 0,
//		 	yflip : attrs&(1<<6) != 0,
//		 	xflip : attrs&(1<<5) != 0,
//		 	palette : (attrs>>3)&1
//		 };
//		 sprites.push(s)
//	}
//	sprites
//}
//
//fn draw_tileset(renderer : &mut Renderer, tileset : Vec<Texture>) {
//	renderer.set_draw_color(Color::RGB(191,191,255));
//	//draw grid	
//	for y in 0..16 {
//		let off = TILE_OFFSET as i32;
//		renderer.draw_line( Point::new(0,y*off-1) , Point::new(WIDTH as i32,y*off-1));
//	}
//	for x in 0..16 {
//		let off = TILE_OFFSET as i32;
//		renderer.draw_line( Point::new(x*off-1,0) , Point::new(x*off-1,HEIGHT as i32));
//	}
//	//draw tiles
//	for t in 0..256 {
//		let screen_y = ((t as u32)%16)*TILE_OFFSET;
//		let screen_x = ((t as u32)/16)* TILE_OFFSET;
//		renderer.copy(&tileset[t], None, Some(Rect::new_unwrap(screen_y as i32, screen_x as i32, TILE_SIZE,TILE_SIZE)));
//	}
//	
//}
