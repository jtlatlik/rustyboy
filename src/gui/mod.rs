extern crate sdl2;
extern crate libc;
//mod tileview;
use std::mem;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};
use std::process;
use std::collections::HashMap;


use system::video::*;
use system::system::GBSystem;
use core::cpu::CPU;

use self::sdl2::Sdl;
use self::sdl2::EventPump;
use self::sdl2::video::Window;
use self::sdl2::render::{self,Renderer,Texture};
use self::sdl2::pixels::PixelFormatEnum;
use self::sdl2::pixels::Color;
use self::sdl2::pixels::Palette;
use self::sdl2::rect::Point;
use self::sdl2::rect::Rect;
use self::sdl2::event::Event;
use self::sdl2::keyboard::Keycode;
use self::sdl2::surface::Surface;
use self::sdl2::controller::{Button, GameController, Axis};

use std::io::Write;
use time;

const FRAME_SAMPLES: u32 = 30;

pub struct GUI<'a> {
	renderer : Renderer<'a>,
	event_pump : EventPump,
	frames : u32,
	controller : Option<GameController>,
	pub break_request: bool,
	pub speed_mode: bool,
	frame_ns: u64,
	fps:f64
}

pub fn init<'a>() -> GUI<'a> {
	
	let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    
    let mut window : Window = video_subsystem.window("Rustyboy", 640, 576)
    	.position_centered()
    	.opengl()
    	.build()
    	.unwrap();
	

	let mut renderer : Renderer = window.renderer().target_texture().build().unwrap();
	
	let mut event_pump = sdl_context.event_pump().unwrap();

	let mut controller = None;
	for i in 0..game_controller_subsystem.num_joysticks().unwrap() {
		
		if game_controller_subsystem.is_game_controller(i) {
            match game_controller_subsystem.open(i) {
                Ok(c) => {
                    // We managed to find and open a game controller,
                    // exit the loop
                    controller = Some(c);
                    break;
                },
                Err(e) => println!("failed: {:?}", e),
            }
		}
	}

	GUI {
		renderer : renderer,
		event_pump : event_pump,
		frames : 0,
		controller : controller,
		break_request : false,
		speed_mode : false,
		frame_ns : time::precise_time_ns(),
		fps : 0.0
	}
}

impl<'a> GUI<'a> {

	pub fn update(&mut self, cpu : &mut CPU) {
		
		self.break_request = false;
		let renderer = &mut self.renderer;
		let event_pump = &mut self.event_pump;
			
		if !cpu.sys.borrow().video.frame_ready {
			return
		}
		
		
		self.frames += 1;
		if self.frames == FRAME_SAMPLES {
			let frame_delta = time::precise_time_ns() - self.frame_ns;
			self.frame_ns += frame_delta;
			self.fps = (1000000000.0*(FRAME_SAMPLES as f64)) / (frame_delta as f64);
			self.frames = 0;		
		}
		{
			let sys = cpu.sys.borrow();
			let scy = sys.video.regs.scy.data;
			let scx = sys.video.regs.scx.data;
			let wx =  sys.video.regs.wx.data;
			let wy =  sys.video.regs.wy.data;
			let window_title = &format!("fps {:.1}; scy={}, scx={}, wy={}, wx={}", self.fps, scy, scx, wy, wx);
			
			
			renderer.window_mut().unwrap().set_title(window_title);
		}
		
		
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                	process::exit(0);
                    //break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                	self.break_request = true;
                },
//                Event::ControllerAxisMotion { axis, value: val, .. } => {
//	                let dead_zone = 5000;
//	                let neg_zone = val < -dead_zone;
//	                let pos_zone = val > dead_zone;
//	                match axis {
//	                	Axis::LeftX => {
//	                		cpu.sys.borrow_mut().joypad.set_left_pressed(neg_zone);
//	                		cpu.sys.borrow_mut().joypad.set_right_pressed(pos_zone)
//	                	},
//	                	Axis::LeftY => {
//	                		cpu.sys.borrow_mut().joypad.set_up_pressed(neg_zone);
//	                		cpu.sys.borrow_mut().joypad.set_down_pressed(pos_zone)
//	                	},
//	                	_ => {}
//	                }
//                },
	            Event::ControllerButtonDown{ button, .. } => { 
	            	match button {
	            		Button::A => cpu.sys.borrow_mut().joypad.set_a_pressed(true),
	            		Button::B => cpu.sys.borrow_mut().joypad.set_b_pressed(true),
	            		Button::Start => cpu.sys.borrow_mut().joypad.set_start_pressed(true),
	            		Button::Back => cpu.sys.borrow_mut().joypad.set_select_pressed(true),
	            		Button::DPadDown => cpu.sys.borrow_mut().joypad.set_down_pressed(true),
	            		Button::DPadLeft => cpu.sys.borrow_mut().joypad.set_left_pressed(true),
	            		Button::DPadUp => cpu.sys.borrow_mut().joypad.set_up_pressed(true),
	            		Button::DPadRight => cpu.sys.borrow_mut().joypad.set_right_pressed(true),
	            		Button::RightShoulder => self.speed_mode = true,
	            		_ => {}
	            	}
	            },
	            Event::ControllerButtonUp{ button, .. } => { 
	            	match button {
	            		Button::A => cpu.sys.borrow_mut().joypad.set_a_pressed(false),
	            		Button::B => cpu.sys.borrow_mut().joypad.set_b_pressed(false),
	            		Button::Start => cpu.sys.borrow_mut().joypad.set_start_pressed(false),
	            		Button::Back => cpu.sys.borrow_mut().joypad.set_select_pressed(false),
	            		Button::DPadDown => cpu.sys.borrow_mut().joypad.set_down_pressed(false),
	            		Button::DPadLeft => cpu.sys.borrow_mut().joypad.set_left_pressed(false),
	            		Button::DPadUp => cpu.sys.borrow_mut().joypad.set_up_pressed(false),
	            		Button::DPadRight => cpu.sys.borrow_mut().joypad.set_right_pressed(false),
	            		Button::RightShoulder => self.speed_mode = false,
	            		_ => {}
	            	}
	            },
                _ => {}
            }
        }
        
    	cpu.sys.borrow_mut().video.frame_ready = false;
        renderer.set_draw_color(Color::RGBA(0,0,255,128));
        renderer.clear();
    	
        let mut tex = renderer.create_texture_streaming(PixelFormatEnum::RGB332, (160, 144)).unwrap();
		tex.with_lock(None, |mut buffer: &mut [u8], pitch: usize| {
				let bb : &[u8; 160*144] = &cpu.sys.borrow().video.back_buffer;
				buffer.write(bb).unwrap();
				
		}).unwrap();	        
        
        renderer.copy(&tex, None, None);
        renderer.present();

	}
}