use bracket_lib::prelude::*;
use specs::*;

use super::game_ui::ItemMenuResult;

pub fn show_selection(ctx: &mut BTerm, title: &str, options: &Vec<(String, Entity)>) {
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
    ctx.print_color(
        18,
        y - 2,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        title,
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    let mut j = 0;
    for (name, _entity) in options
    {
        ctx.set(17, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            97 + j as FontCharType,
        );
        ctx.set(19, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(')'));

        ctx.print(21, y, name);
        y += 1;
        j += 1;
    }
}

pub fn read_input_selection(key: Option<VirtualKeyCode>, options: &Vec<(String, Entity)>) -> ItemMenuResult<Entity> {
    let count = options.len();

    match key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return ItemMenuResult::Selected { result: options[selection as usize].1 };
                }
                ItemMenuResult::NoResponse
            }
        },
    }
}