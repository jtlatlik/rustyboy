use std::thread;
use super::sdl2::pixels::PixelFormatEnum;
use super::sdl2::pixels::Color;
use super::sdl2::rect::Point;
use super::sdl2::rect::Rect;
use super::sdl2::event::Event;
use super::sdl2::keyboard::Keycode;
use system::system::GBSystem;
use std::sync::{Arc, RwLock};


use system::video::*;
//use super::sdl2::render::{Texture,Renderer, RenderTarget};
use super::sdl2::render::*;

use system::*;

const SCALE : u32 = 4;
const TILE_SIZE :u32 = SCALE*8;
const TILE_OFFSET :u32 = TILE_SIZE+1; 

const WIDTH : u32 = TILE_OFFSET*16 - 1;
const HEIGHT : u32 = TILE_OFFSET*16 - 1;

fn draw_tileset(renderer : &mut Renderer, tileset : Vec<Texture>) {
	renderer.set_draw_color(Color::RGB(191,191,255));
	//draw grid	
	for y in 0..16 {
		let off = TILE_OFFSET as i32;
		renderer.draw_line( Point::new(0,y*off-1) , Point::new(WIDTH as i32,y*off-1));
	}
	for x in 0..16 {
		let off = TILE_OFFSET as i32;
		renderer.draw_line( Point::new(x*off-1,0) , Point::new(x*off-1,HEIGHT as i32));
	}
	//draw tiles
	for t in 0..256 {
		let screen_y = ((t as u32)%16)*TILE_OFFSET;
		let screen_x = ((t as u32)/16)* TILE_OFFSET;
		renderer.copy(&tileset[t], None, Some(Rect::new_unwrap(screen_y as i32, screen_x as i32, TILE_SIZE,TILE_SIZE)));
	}
	
}

pub fn display(sys : ThreadSafeSystem) {
	
    let sdl_context = super::sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("Tile Viewer", WIDTH*2, HEIGHT)
    	.position_centered()
    	.opengl()
    	.build()
    	.unwrap();
    	
	let mut renderer = window.renderer().target_texture().build().unwrap();

    	let mut event_pump = sdl_context.event_pump().unwrap();

	    'running: loop {
	        for event in event_pump.poll_iter() {
	            match event {
	                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
	                    break 'running
	                },
	                _ => {}
	            }
	        }
	        
	        let mut palette = [Color::RGB(0,0,0); 4];
			
			renderer.set_draw_color(Color::RGB(0,0,0));
			renderer.clear();
	        
    		let mut tileset : Vec<Texture> = Vec::new();
	        //acquire read lock and read tile data
			{
				let lock = sys.read().unwrap();
				let video = &(*lock).video;

				for i in 0..4 {
					let c = (*video.regs.bgp >> (2*i)) & 0x3;
					palette[i] = match c {
						0 => Color::RGB(255,255,255),
						1 => Color::RGB(192,192,192),
						2 => Color::RGB(96,96,96),
						3 => Color::RGB(0,0,0),
						_ => unreachable!()
					}
				}

				let tileset_addr = if ((*video.regs.lcd_ctrl >> LCD_CONTROL_TSSEL) & 1) != 0 { 0 } else { 0x800 };
				
				for t  in 0..256  {
					
					tileset.push(renderer.create_texture_streaming(PixelFormatEnum::RGB24, (8,8)).unwrap());
					tileset[t].with_lock(None, |buffer: &mut [u8], pitch :usize| {
						let tile_addr = tileset_addr as usize + t*16;
						for y in 0..8 {
							let tile_data = [ video.vram0[tile_addr + y*2],  video.vram0[tile_addr + y*2 + 1]];
							for x in 0..8 {
								let index = (((tile_data[1] >> (7-x))&1) << 1) | ((tile_data[0] >> (7-x))&1);
								if let Color::RGB(r,g,b) = palette[index as usize] {
									let offset = y*pitch + x*3;
									buffer[offset +0] = r;
									buffer[offset +1] = g;
									buffer[offset +2] = b;
								}
								
							}
						}
					}).unwrap();
				}
				
				
				//now draw background with tile data
				let tilemap_addr = 0x1800 + (((*video.regs.lcd_ctrl >> LCD_CONTROL_BGMAP) & 1)  as u16)*0x400; 
				
				renderer.render_target().unwrap().create_and_set(PixelFormatEnum::RGB24, (256,256));
				renderer.clear();
				//render into new target texture
				for y in 0..32 {
					for x in 0..32 {
						let offset : u16 = (y as u16)*32+(x as u16);
						let index = video.vram0[(tilemap_addr + offset) as usize] as usize;
						
						renderer.copy(&tileset[index], None, Some(Rect::new_unwrap((x as i32)*8,(y as i32)*8,8,8)));
					}
				}		
			}
			//renderer.present();
			
			
			match renderer.render_target().unwrap().reset().unwrap() {
				
				Some(tex) => {
					renderer.copy(&tex, None, Some(Rect::new_unwrap( WIDTH as i32, 0, WIDTH  , HEIGHT)));
				}
				None => ()
			}
			
			draw_tileset(&mut renderer, tileset);
			renderer.present();
			
	        thread::sleep_ms(100)
	    }
}