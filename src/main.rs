use cursive::views::{ScrollView, TextView, TextContent};
use clap::{App, Arg};
use std::{io, thread, marker, fs};
use std::sync::{Arc, atomic};

// A single UTF-8 character occupies between 1 and 4 bytes. Thus this is 4.
const BUFFER_SIZE: usize = 4;

unsafe fn spawn_inp_channel<F>(mut file: F, stop: Arc<atomic::AtomicBool>, content: TextContent)
where
    F: io::Read,
    F: marker::Send + 'static,
{

    /*
    The block below instantiates a thread and iterates over the file to read its content

    Keywords:
        `move`: Ensures that the spawned thread has ownership of the variables it uses.
        `loop`: Ensures that the file is continiously looped over.
    */
    thread::spawn(move || {
        // Create two vectors that will contain what was left from the previous read and what is from the current read respectively.
        let mut remainder: Vec<u8> = Vec::new();
        let mut buffer: Vec<u8> = Vec::new();
        
        // Fill the buffer so that read() will be able to place characters there initially
        while buffer.len() < BUFFER_SIZE {
            buffer.push(u8::MAX);
        }

        loop {
            // Get a mutable reference to a shared boolean between the threads in order to terminate the Thread at a later date
            if stop.load(atomic::Ordering::Relaxed) {
                break;
            }
            // If the read was successful, return a value between 0 and BUFFER_SIZE. If it was interruted or smth return 0
            let s = file.read(&mut buffer[..]).unwrap_or(0);


            if s > 0 {
                // Add the contents of `buffer` to the remainder from last iteration in order to correctly identify any characters who's bytes were split in the read
                remainder.append(&mut buffer.clone());

                // Create a string and get all the valid UTF-8 characters from it. The rest are replaced with `U+FFFD`
                let mut _str = String::new();
                _str.push_str(&String::from_utf8_lossy(&remainder));

                // Retrieve the last index of an invalid character. This assumes that the text only contains valid characters. 
                let last_index = &_str.rfind("\u{FFFD}").unwrap_or(0);

                // Remove all the characters after the last `U+FFFD` as they are bytes who will be cleared up in the next iteration
                _str.drain(last_index..);

                // Remove all the bytes up to the last valid character to prevent reprinting of characters.
                remainder.drain(0..*last_index);

                // Expand the current content of the display area
                let current_content = content.get_content().source().to_string();
                content.set_content(format!("{}{}", current_content, _str));

            }
        }
    });
}

fn main() {
    // Parse commandline arguments
    let args = App::new("Reed")
                    .version("0.1")
                    .about("Less but rust")
                    .args(
                        &[
                            Arg::with_name("File")
                                .index(1)
                                .about("The inputted file into the program")
                        ]
                    ).get_matches();

    // Creates the cursive root - required for every application.
    let mut siv = cursive::default();

    let content = TextContent::new("");
    let view = TextView::new_with_content(content.clone());
    let scroll = ScrollView::new(view);

    siv.add_global_callback('q', |s| s.quit());
    siv.add_fullscreen_layer(scroll);

    // Sets the refresh rate of the items displayed to 30 fps as otherwise it will block and nothing will be displayed.
    siv.set_autorefresh(true);

    // Create a shared reference to a thread-safe boolean which will inform the other threads to terminate
    let stop_thread = Arc::new(atomic::AtomicBool::new(false));

    // This is needed to accept input as an open pipe or as a path to a file
    unsafe {
        if args.is_present("File") {
            let path = args.value_of("File").unwrap();
            let file = fs::File::open(path).unwrap();

            // The `content` reference is passed so that the thread itself can append the text it gets.
            spawn_inp_channel(file, stop_thread.clone(), content.clone());
        } else {

            // The `content` reference is passed so that the thread itself can append the text it gets.
            spawn_inp_channel(io::stdin(), stop_thread.clone(), content.clone());
        }
    }

    // Starts the event loop.
    siv.run();
    stop_thread.store(true, atomic::Ordering::SeqCst);
}