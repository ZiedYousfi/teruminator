use crossterm::{
    cursor::position,
    terminal::size,
};

fn usable_space() -> std::io::Result<(u16, u16)> {
    let (cols, rows) = size()?;
    let (cur_col, cur_row) = position()?;

    let lines_below = rows.saturating_sub(cur_row + 1);
    let cols_avail = cols.saturating_sub(cur_col);

    Ok((cols_avail, lines_below))
}

fn main() -> std::io::Result<()> {
    let (mut cols, mut lines);

    let mut last_render_time = std::time::Instant::now();

    loop {

        (cols, lines) = usable_space()?;
        let total_cells = cols as usize * lines as usize;

        let screen: String = (0..(total_cells))
        .map(|_| "e")

            .collect();
        print!("\r{}", screen);

        print!(
            "\r --- Temps écoulé depuis le dernier rendu : {:?} ms ---",
            last_render_time.elapsed().as_millis()
        );

        std::io::Write::flush(&mut std::io::stdout())?;
        last_render_time = std::time::Instant::now();
    }
}