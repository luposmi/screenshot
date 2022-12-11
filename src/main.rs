#![feature(vec_into_raw_parts)]

use x11::xlib;

use std::{time::Instant, vec};

use image::ImageBuffer;
//use image::data::ImageData;

use std::ffi::c_uchar;
use std::thread;

//let mut last_call = Instant::now();

use image::Rgb;
fn main() {
    let mut total= Instant::now();
    let mut ms=Instant::now();
    // Open a connection to the X11 server
    let display = unsafe { xlib::XOpenDisplay(std::ptr::null()) };

    // Get the root window
    let root_window = unsafe { xlib::XDefaultRootWindow(display) };

    // Get the dimensions of the root window
    let width = unsafe{ xlib::XDisplayWidth(display, 0) as usize };
    let height = unsafe{ xlib::XDisplayHeight(display, 0) as usize };

    ms = printtimedif(ms,"Startup X:");

    // Take the screenshot
    let ximage: *mut xlib::XImage = unsafe {
        xlib::XGetImage(
            display,
            root_window,
            0,
            0,
            width as u32,
            height as u32,
            xlib::XAllPlanes(),
            xlib::ZPixmap,
        )
    };

    ms = printtimedif(ms,"XGetImage:");

    // Create an image buffer to store the screenshot
    let mut vec_buffer: Vec<u8> = vec![0 as u8;width*height*3];
    
    // Disassemble the buffer
    let (vec_ptr, vec_length, vec_capacity) = vec_buffer.into_raw_parts();

    

    let ximage_data_ptr = unsafe{((*ximage).data as *const i32) as usize};
    let vec_buffer_ptr = vec_ptr as usize;
    let threadnum=4;


    let mut threads = Vec::new();
    // Write the slice into the buffer. Note that the slice has the format i32 while the buffer wants u8 
    // The is c in rust syntax; it needs to be performant; does not decrease security
    for i in 0..threadnum {
        //let data_ptr = data_ptr.clone();

        threads.push(thread::spawn(move || {
            for x in (width/threadnum*i)..(width/threadnum*(i+1)) {
                for y in 0..height {
                    let (r,g,b) = {
                        let data_ptr: *const i32 = unsafe{ std::mem::transmute(ximage_data_ptr)};
                        let pixel = unsafe {*data_ptr.offset((y * width + x) as isize)};
                        (
                            (pixel >> 16) as c_uchar,
                            (pixel >> 8) as c_uchar,
                            pixel as c_uchar,
                        )
                    };
                    let offset=((y * width + x)*3) as usize;
                    unsafe{
                        std::ptr::write(std::mem::transmute(vec_buffer_ptr+offset), r);
                        std::ptr::write(std::mem::transmute(vec_buffer_ptr+offset+1), g);
                        std::ptr::write(std::mem::transmute(vec_buffer_ptr+offset+2), b);
                    }
                }
            }
        }));
    }

    // Wait for all threads to finish
    for thread in threads {
        thread.join().unwrap();
    }
    
    // Reassamble the buffer
    vec_buffer=unsafe {Vec::from_raw_parts(vec_ptr, vec_length, vec_capacity)};

    // Write the buffer into an imagebuffer
    let image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(width as u32, height as u32, vec_buffer).unwrap();

    ms = printtimedif(ms,"write ImageBuffer:");

    // Save the screenshot as a JPEG
    image_buffer.save("screenshot.jpg").unwrap();

    ms = printtimedif(ms,"save jpg:");

    // Clean up
    unsafe {
        xlib::XDestroyImage(ximage);
        xlib::XCloseDisplay(display);
    }

    printtimedif(ms,"clean up:");
    printtimedif(total,"total time:");
}


// Print the time each step takes
#[cfg(feature = "printtime")]
fn printtimedif(last_call: Instant, message: &str)-> Instant{
    let now = Instant::now();
    let time_diff = now.duration_since(last_call);
    println!("{}\n\t {} us.",message,time_diff.as_micros());
    now
}

#[cfg(not(feature = "printtime"))]
#[allow(unused_variables)]
fn printtimedif(last_call: Instant, message: &str)-> Instant{last_call}
