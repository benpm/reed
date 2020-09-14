use cursive::views::{ScrollView, TextView, TextContent};
use clap::{App, Arg};
use std::{io, thread, marker, fs};
use std::sync::mpsc;

unsafe fn spawn_inp_channel<F>(mut file: F) -> mpsc::Receiver<String>
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
        let mut buffer = String::new();
        if let Err(_s) = file.read(buffer.as_mut_vec()) {
            panic!("Issue arose in channel thread.")
        }
        tx.send(buffer).unwrap();
    });

    rx
}

fn main() {
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

    let content = TextContent::new("");
    let view = TextView::new_with_content(content.clone());
    let scroll = ScrollView::new(view);

    let mut siv = cursive::default();

    siv.add_global_callback('q', |s| s.quit());
    siv.add_fullscreen_layer(scroll);

    let rx: mpsc::Receiver<String>;

    unsafe {
        if args.is_present("File") {
            let path = args.value_of("File").unwrap();
            let file = fs::File::open(path).unwrap();
            rx =  spawn_inp_channel(file);
        } else {
            rx =  spawn_inp_channel(io::stdin());
        }
    }

    // TODO: Figure out whenever the content of the file updates and read the next part of the queue onto the screen automatically.

    loop {
        match rx.try_recv() {
            Ok(s) => {
                content.set_content(s);
                siv.step();
            },
            Err(mpsc::TryRecvError::Empty) => {},
            Err(mpsc::TryRecvError::Disconnected) => {},
        }
    }

    // // Starts the event loop.
    // siv.run();
}