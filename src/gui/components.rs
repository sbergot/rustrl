use bracket_lib::prelude::*;

pub fn show_selection<T>(ctx: &mut BTerm, title: &str, options: &Vec<(String, T)>) {
    let count = options.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    ctx.print_color(18, y - 2, RGB::named(YELLOW), RGB::named(BLACK), title);
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    let mut j = 0;
    for (name, _entity) in options {
        ctx.print(18, y, format_option(j, name));
        y += 1;
        j += 1;
    }
}

pub fn option_to_letter(i: usize) -> char {
    ("abcdefghijklmnopqrstuvwxyz").chars().nth(i).unwrap()
}

pub fn format_option(idx: usize, label: &str) -> String {
    format!("({}) {}", option_to_letter(idx), label)
}