
#![no_std]
#[allow(static_mut_refs)]
extern crate alloc;

#[macro_use]
extern crate playdate as pd;

use core::ffi::*;
use core::ptr::NonNull;
use alloc::ffi::CString;
use alloc::vec::Vec;
use no_std_io2::io;
use no_std_io2::io::Cursor;
use no_std_io2::io::Read;
use no_std_io2::io::Seek;
use no_std_io2::io::SeekFrom;
use no_std_io2::io::Write;
use pd::fs::FileOptions;
use pd::sys::ffi::SDFile;
use pd::sys::EventLoopCtrl;
use pd::sys::ffi::PlaydateAPI;
use pd::system::update::UpdateCtrl;
use pd::display::Display;
use pd::graphics::*;
use pd::graphics::text::*;
use pd::graphics::bitmap::*;
use pd::system::prelude::*;
use pd::sound::prelude::*;
use pd::fs::Path;
use tiled::Loader;
use tiled::ResourcePath;
use tiled::ResourceReader;
// use xml::EventReader;


/// Game state
struct State {
	// TODO: Fill the state
}


impl State {
	fn new() -> Self {
		// TODO: Init the state

		Self {}
	}


	/// System event handler
	fn event(&'static mut self, event: SystemEvent) -> EventLoopCtrl {
		match event {
			// Initial setup
			SystemEvent::Init => {
				// Set FPS to 30
				Display::Default().set_refresh_rate(30.0);

				// Register our update handler that defined below
				self.set_update_handler();

				println!("Game init complete");
			},
			// TODO: React to other events
			_ => {},
		}
		EventLoopCtrl::Continue
	}
}


impl Update for State {
	/// Updates the state
	fn update(&mut self) -> UpdateCtrl {
		clear(Color::WHITE);


		// TODO: update the state of game


		System::Default().draw_fps(0, 0);

		UpdateCtrl::Continue
	}
}


/// Entry point
#[unsafe(no_mangle)]
pub fn event_handler(_api: NonNull<PlaydateAPI>, event: SystemEvent, _sim_key_code: u32) -> EventLoopCtrl {
	// Unsafe static storage for our state.
	// Usually it's safe because there's only one thread.
	// let mut reader = EventReader::new(FileHandle::read_only("assets/tiles.tsx").unwrap()).into_iter();
	System::Default().set_update_callback_boxed(move |_| {

		clear(Color::WHITE);

		let mut loader = Loader::with_reader(PDTiledReader);
		let map = loader.load_tmx_map("assets/test-map.tmx");
		println!("{:?}", map);
		// let next = reader.next();
		// println!("{:?}", next);
		// if matches!(next, None | Some(Err(_)) | Some(Ok(xml::reader::XmlEvent::EndDocument))) {
		// 	println!("finished");
		// 	// reader = EventReader::new(FileHandle::read_only("assets/tiles.tsx").unwrap()).into_iter();
		// }
		// TODO: update the state of game


		System::Default().draw_fps(0, 0);
		UpdateCtrl::Continue
	}, ());

	EventLoopCtrl::Stop
	// pub static mut STATE: Option<State> = None;
	// if unsafe { STATE.is_none() } {
	// 	let state = State::new();
	// 	unsafe { STATE = Some(state) }
	// }

	// // Call state.event
	// unsafe { STATE.as_mut().expect("impossible") }.event(event)
}

pub struct PDTiledReader;

impl ResourceReader for PDTiledReader {
    type Resource = Cursor<Vec<u8>>;
    type Error = no_std_io2::io::Error;

    fn read_from(&mut self, path: &ResourcePath) -> Result<Self::Resource, Self::Error> {
        println!("requested file: {path}");
        let mut buffer = FileHandle::read_only(path)?;
        let mut out_vec = Vec::new();
        buffer.read_to_end(&mut out_vec)?;
        Ok(Cursor::new(out_vec))
        
        // FileHandle::read_only(path)
    }
}


pub struct FileHandle {
    handle: *mut SDFile,
}

impl FileHandle {
    /// Opens a handle for the file at path.
    ///
    /// The kFileRead mode opens a file in the game pdx, while kFileReadData searches
    /// the game’s data folder; to search the data folder first then fall back on the game pdx,
    /// use the bitwise combination kFileRead|kFileReadData.
    /// kFileWrite and kFileAppend always write to the data folder.
    ///
    /// The function returns Err if the path contains a \0 byte (see [`CString::new`]).
    /// The function returns Err if the file at path cannot be opened, and will log the error to the console.
    /// The filesystem has a limit of 64 simultaneous open files.
    pub fn open(path: &str, mode: FileOptions) -> io::Result<Self> {
        let c_path = CString::new(path)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;
		println!("fn ptr: {:p}", pd::sys::api!(file).open.unwrap());
        let handle = unsafe { pd::sys::api!(file).open.unwrap()(c_path.as_ptr(), mode) };
        if handle.is_null() {
            let message = unsafe { pd::sys::api!(file).geterr.unwrap()() };
            unsafe { pd::sys::api!(system).logToConsole.unwrap()(message) };

            Err(io::Error::new(io::ErrorKind::Other, "Failed to open file"))
        } else {
			println!("open file: {:p}", handle);
			Ok(FileHandle { handle })
        }
    }

    /// Opens a handle for a file at path.
    /// Shorthand for [`Self::open`] with kFileRead and kFileReadData
    #[inline]
    pub fn read_only(path: &str) -> io::Result<Self> {
        Self::open(path, FileOptions::kFileRead | FileOptions::kFileReadData)
    }

    /// Opens a handle for a file at path.
    /// Shorthand for [`Self::open`] with kFileWrite (and kFileAppend if append = true)
    #[inline]
    pub fn write_only(path: &str, append: bool) -> io::Result<Self> {
        let append = if append {
            FileOptions::kFileAppend
        } else {
            FileOptions(0)
        };
        Self::open(path, FileOptions::kFileWrite | append)
    }

    /// Opens a handle for a file at path.
    /// Shorthand for [`Self::open`] with kFileRead, kFileReadData, kFileWrite (and kFileAppend if append = true)
    #[inline]
    pub fn read_write(path: &str, append: bool) -> io::Result<Self> {
        let append = if append {
            FileOptions::kFileAppend
        } else {
            FileOptions(0)
        };
        Self::open(
            path,
            FileOptions::kFileRead | FileOptions::kFileReadData | FileOptions::kFileWrite | append,
        )
    }
}

impl Read for FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		println!("read");
        let result = unsafe {
            pd::sys::api!(file).read.unwrap()(
                self.handle,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
            )
        };
        // println!("read: {}", result);
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Read error"))
        } else {
            Ok(result as usize)
        }
    }
}

impl Write for FileHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = unsafe {
            pd::sys::api!(file).write.unwrap()(
                self.handle,
                buf.as_ptr() as *const c_void,
                buf.len() as u32,
            )
        };
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Write error"))
        } else {
            Ok(result as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = unsafe { pd::sys::api!(file).flush.unwrap()(self.handle) };
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Flush error"))
        } else {
            Ok(())
        }
    }
}

impl Seek for FileHandle {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Start(n) => (n as c_int, 0),
            SeekFrom::End(n) => (n as c_int, 2),
            SeekFrom::Current(n) => (n as c_int, 1),
        };
        let result = unsafe { pd::sys::api!(file).seek.unwrap()(self.handle, offset, whence) };
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Seek error"))
        } else {
            Ok(result as u64)
        }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
		println!("file dropped: {:p}", self.handle);
        unsafe { pd::sys::api!(file).close.unwrap()(self.handle) };
    }
}


// Needed for debug build, absolutely optional
ll_symbols!();
