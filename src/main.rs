use cursive::views::{ScrollView, TextView, TextContent};

fn main() {
    // Creates the cursive root - required for every application.
    let mut siv = cursive::default();

    siv.add_global_callback('q', |s| s.quit());

    let text_area = TextContent::new("This is a test");
    let view = TextView::new_with_content(text_area.clone());
    let scroll = ScrollView::new(view);
    siv.add_fullscreen_layer(scroll);
    text_area.set_content("AWNAWNAOROWA");

    // Starts the event loop.
    siv.run();
}