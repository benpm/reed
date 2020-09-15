use cursive::views::{ScrollView, TextView, TextContent};
use clap::{App, Arg};
use std::{io, thread, marker, fs};
use std::sync::{Arc, atomic};

const BUFFER_SIZE: usize = 128;

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
    thread::spawn(move || loop {
        // Get a mutable reference to a shared boolean between the threads in order to terminate the Thread at a later date
        if stop.load(atomic::Ordering::Relaxed) {
            break;
        }
        let buffer: &mut [u8; BUFFER_SIZE] = &mut [u8::MAX; BUFFER_SIZE];
        match file.read(buffer) {
            Ok(s) => {
                if s != 0 {
                    let current_content = content.get_content().source().to_string();
                    content.set_content(format!("{}{}", current_content, String::from_utf8(buffer[0..s].to_vec()).unwrap()));
                }
            },
            Err(_) => panic!("Issue arose in channel thread.")
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
    siv.set_autorefresh(true);

    // Create a shared reference to a thread-safe boolean which will inform the other threads to terminate
    let stop_thread = Arc::new(atomic::AtomicBool::new(false));

    // This is needed to accept input as an open pipe or as a path to a file
    unsafe {
        if args.is_present("File") {
            let path = args.value_of("File").unwrap();
            let file = fs::File::open(path).unwrap();
            spawn_inp_channel(file, stop_thread.clone(), content.clone());
        } else {
            spawn_inp_channel(io::stdin(), stop_thread.clone(), content.clone());
        }
    }

    // Starts the event loop.
    siv.run();
    stop_thread.store(true, atomic::Ordering::SeqCst);
}