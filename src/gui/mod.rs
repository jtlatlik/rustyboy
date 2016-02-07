extern crate sdl2;

mod tileview;

use std::thread;
use std::thread::JoinHandle;

use system::*;

pub fn init(sys : ThreadSafeSystem)  -> JoinHandle<()> {
	thread::spawn(move || {

		tileview::display(sys)
    })
}
