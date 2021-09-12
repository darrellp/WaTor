extern crate cursive;
#[macro_use]
extern crate lazy_static;
extern crate pancurses;
mod board;
mod wator_cursive;

fn main() {
    let mut siv = cursive::default();
    wator_cursive::wator_cursive::setup(&mut siv);
    siv.run()
}
