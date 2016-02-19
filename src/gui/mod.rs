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
use self::sdl2::controller::{Button, GameController};

use std::io::Write;


pub struct GUI<'a> {
	renderer : Renderer<'a>,
	event_pump : EventPump,
	frames : u32,
	controller : Option<GameController>
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
		controller : controller
	}
}

impl<'a> GUI<'a> {

	pub fn update(&mut self, cpu : &mut CPU) {
		
		//thread::spawn(move || {
		let renderer = &mut self.renderer;
		let event_pump = &mut self.event_pump;

//				cpu.run_instruction();
			
		if !cpu.sys.borrow().video.frame_ready {
			return
		}
		
		self.frames+=1;
		
		
		renderer.window_mut().unwrap().set_title(&format!("frame {}", self.frames));
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                	process::exit(0);
                    //break 'running
                },
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