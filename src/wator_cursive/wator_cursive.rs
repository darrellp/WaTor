use crate::board::board::{Board, Cell};
use crate::cursive;
use cursive::theme::{BaseColor, Color, ColorStyle};
use cursive::traits::Boxable;
use cursive::views::{Canvas, LinearLayout, TextView};
use cursive::Printer;

pub const WIDTH: usize = 70;
pub const HEIGHT: usize = 10;

lazy_static! {
    static ref BOARD: Board<WIDTH, HEIGHT> = {
        #[allow(unused_mut)]
        let mut b = Board::<WIDTH, HEIGHT>::new(0.4, 0.4, 2, 2, 3, 2);
        b
    };
}

pub fn setup(app: &mut cursive::CursiveRunnable) {
    app.add_global_callback('q', |s| s.quit());
    let mut panes = LinearLayout::vertical();
    panes.add_child(Canvas::new(()).with_draw(draw).fixed_size((WIDTH, HEIGHT)));
    panes.add_child(TextView::new("Controls").fixed_size((WIDTH, 1)));
    app.add_layer(panes);
}

fn draw(_: &(), p: &Printer) {
    let width = p.size.x;
    let height = p.size.y;

    for row in 0..height {
        for col in 0..width {
            let (next_ch, front_color) = match BOARD.cell_at(row, col) {
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
}
