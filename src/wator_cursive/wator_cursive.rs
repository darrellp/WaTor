use crate::board::board::{Board, Cell};
use crate::cursive;
use cursive::theme::{BaseColor, Color, ColorStyle};
use cursive::traits::Boxable;
use cursive::views::{Canvas, LinearLayout, TextView};
use cursive::Printer;
use std::sync::Mutex;

pub const WIDTH: usize = 90;
pub const HEIGHT: usize = 25;

lazy_static! {
    static ref BOARD: Mutex<Board<WIDTH, HEIGHT>> = {
        #[allow(unused_mut)]
        let mut b = Board::<WIDTH, HEIGHT>::new(0.2, 0.01, 20, 5, 200, 1);
        Mutex::new(b)
    };
}

fn advance() {
    BOARD.lock().unwrap().advance();
}

fn cell_at(row: usize, col: usize) -> Cell {
    BOARD.lock().unwrap().cell_at(row, col)
}

pub fn run(app: &mut cursive::CursiveRunnable) {
    let mut panes = LinearLayout::vertical();
    let canvas = Canvas::new(()).with_draw(draw).fixed_size((WIDTH, HEIGHT));
    panes.add_child(canvas);
    panes.add_child(TextView::new("Controls").fixed_size((WIDTH, 1)));
    app.add_layer(panes);
    app.set_fps(20);
}

fn draw(_: &(), p: &Printer) {
    let width = p.size.x;
    let height = p.size.y;

    for row in 0..height {
        for col in 0..width {
            let (next_ch, front_color) = match cell_at(row, col) {
                Cell::Shark(_, _, _) => ("S", Color::Dark(BaseColor::Red)),
                Cell::Fish(_, _) => ("F", Color::Light(BaseColor::Green)),
                _ => (" ", Color::Dark(BaseColor::Black)),
            };
            let style = ColorStyle::new(front_color, Color::Dark(BaseColor::Black));
            p.with_color(style, |printer| {
                // col is printer's x, row is printer's y
                printer.print((col, row), next_ch);
            })
        }
    }
    advance();
}
