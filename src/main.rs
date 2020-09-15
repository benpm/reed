use cursive::views::{ScrollView, TextView, TextContent};
use clap::{App, Arg};
use std::{io, thread, marker, fs};
use std::sync::{Arc, atomic, mpsc};

unsafe fn spawn_inp_channel<F>(mut file: F, stop: Arc<atomic::AtomicBool>) -> mpsc::Receiver<String>
where
    F: io::Read,
    F: marker::Send + 'static,
{
    // Create a FIFO queue in order to read the contents of the input
    let (tx, rx) = mpsc::channel::<String>();
    /*
    The block below instantiates a thread and iterates over the file to read its content

    Keywords:
        `move`: Ensures that the spawned thread has ownership of the variables it uses.
        `loop`: Ensures that the file is continiously looped over.
    */
    thread::spawn(move || loop {
        // Get a mutable reference to a shared boolean between the threads so that if
        if stop.load(atomic::Ordering::Relaxed) {
            break;
        }
        let mut buffer = String::new();
        if let Err(_s) = file.read(buffer.as_mut_vec()) {
            panic!("Issue arose in channel thread.")
        }
        tx.send(buffer).unwrap();
    });

    rx
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

    // Instantiate a variable that can contain the front end of the FIFO
    let _rx: mpsc::Receiver<String>;

    // Create a shared reference to a thread-safe boolean which will inform the other threads to terminate
    let stop_thread = Arc::new(atomic::AtomicBool::new(false));

    unsafe {
        if args.is_present("File") {
            let path = args.value_of("File").unwrap();
            let file = fs::File::open(path).unwrap();
            _rx = spawn_inp_channel(file, stop_thread.clone());
        } else {
            _rx = spawn_inp_channel(io::stdin(), stop_thread.clone());
        }
    }

    // TODO: Figure out whenever the content of the file updates and read the next part of the queue onto the screen automatically.

    // loop {
    //     match rx.try_recv() {
    //         Ok(s) => {
    //             content.set_content(s);
    //             siv.step();
    //         },
    //         Err(mpsc::TryRecvError::Empty) => {},
    //         Err(mpsc::TryRecvError::Disconnected) => {},
    //     }
    // }

    // // Starts the event loop.
    siv.run();
    stop_thread.store(true, atomic::Ordering::SeqCst);
}