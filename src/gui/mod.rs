extern crate sdl2;

mod tileview;

use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;

use system::system::GBSystem;

pub fn init(sys : &GBSystem)  -> JoinHandle<()> {
	
	let VideoData = sys.video.clone();
	
	thread::spawn(move || {

		tileview::display(VideoData)
    })
}
