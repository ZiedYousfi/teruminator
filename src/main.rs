use crossterm::{
    cursor::position,
    terminal::size,
    style::{Color, SetForegroundColor, ResetColor},
};

fn usable_space() -> std::io::Result<(u16, u16)> {
    let (cols, rows) = size()?;
    let (cur_col, cur_row) = position()?;

    let lines_below = rows.saturating_sub(cur_row + 1);
    let cols_avail = cols.saturating_sub(cur_col);

    Ok((cols_avail, lines_below))
}

// Cube vertices (unit cube centered at origin)
const CUBE_VERTICES: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [-1.0,  1.0,  1.0],
];

// Cube edges (pairs of vertex indices)
const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 2), (2, 3), (3, 0), // back face
    (4, 5), (5, 6), (6, 7), (7, 4), // front face
    (0, 4), (1, 5), (2, 6), (3, 7), // connecting edges
];

// Face definitions for coloring (4 vertices per face, with color)
const CUBE_FACES: [([usize; 4], Color); 6] = [
    ([0, 1, 2, 3], Color::Red),      // back
    ([4, 5, 6, 7], Color::Green),    // front
    ([0, 4, 7, 3], Color::Blue),     // left
    ([1, 5, 6, 2], Color::Yellow),   // right
    ([3, 2, 6, 7], Color::Magenta),  // top
    ([0, 1, 5, 4], Color::Cyan),     // bottom
];

fn rotate_x(point: [f32; 3], angle: f32) -> [f32; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [
        point[0],
        point[1] * cos_a - point[2] * sin_a,
        point[1] * sin_a + point[2] * cos_a,
    ]
}

fn rotate_y(point: [f32; 3], angle: f32) -> [f32; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [
        point[0] * cos_a + point[2] * sin_a,
        point[1],
        -point[0] * sin_a + point[2] * cos_a,
    ]
}

fn rotate_z(point: [f32; 3], angle: f32) -> [f32; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [
        point[0] * cos_a - point[1] * sin_a,
        point[0] * sin_a + point[1] * cos_a,
        point[2],
    ]
}

fn project(point: [f32; 3], width: usize, height: usize, fov: f32, distance: f32) -> Option<(i32, i32, f32)> {
    let z = point[2] + distance;
    if z <= 0.1 { return None; }

    let factor = fov / z;
    let x = (point[0] * factor * 2.0) + (width as f32 / 2.0); // *2 for aspect ratio correction
    let y = (-point[1] * factor) + (height as f32 / 2.0);

    Some((x as i32, y as i32, z))
}

// Bresenham's line algorithm
fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, buffer: &mut Vec<(i32, i32, f32, char, Color)>, z: f32, color: Color) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        buffer.push((x, y, z, '#', color));

        if x == x1 && y == y1 { break; }

        let e2 = 2 * err;
        if e2 >= dy {
            if x == x1 { break; }
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 { break; }
            err += dx;
            y += sy;
        }
    }
}

// Simple face filling using scanlines
#[allow(clippy::too_many_arguments)]
fn fill_face(vertices: &[[f32; 3]; 4], width: usize, height: usize, fov: f32, distance: f32,
             buffer: &mut Vec<(i32, i32, f32, char, Color)>, color: Color, shade_char: char) {
    let mut projected: Vec<(i32, i32, f32)> = Vec::new();

    for v in vertices {
        if let Some(p) = project(*v, width, height, fov, distance) {
            projected.push(p);
        }
    }

    if projected.len() < 3 { return; }

    // Get bounding box
    let min_x = projected.iter().map(|p| p.0).min().unwrap_or(0);
    let max_x = projected.iter().map(|p| p.0).max().unwrap_or(0);
    let min_y = projected.iter().map(|p| p.1).min().unwrap_or(0);
    let max_y = projected.iter().map(|p| p.1).max().unwrap_or(0);

    let avg_z: f32 = projected.iter().map(|p| p.2).sum::<f32>() / projected.len() as f32;

    // Simple point-in-polygon for quad
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_quad(x, y, &projected) {
                buffer.push((x, y, avg_z, shade_char, color));
            }
        }
    }
}

fn point_in_quad(px: i32, py: i32, vertices: &[(i32, i32, f32)]) -> bool {
    if vertices.len() < 3 { return false; }

    let mut inside = true;
    let n = vertices.len();

    for i in 0..n {
        let j = (i + 1) % n;
        let edge_x = vertices[j].0 - vertices[i].0;
        let edge_y = vertices[j].1 - vertices[i].1;
        let point_x = px - vertices[i].0;
        let point_y = py - vertices[i].1;

        let cross = edge_x * point_y - edge_y * point_x;
        if cross < 0 {
            inside = false;
            break;
        }
    }

    if inside { return true; }

    // Try other winding
    inside = true;
    for i in 0..n {
        let j = (i + 1) % n;
        let edge_x = vertices[j].0 - vertices[i].0;
        let edge_y = vertices[j].1 - vertices[i].1;
        let point_x = px - vertices[i].0;
        let point_y = py - vertices[i].1;

        let cross = edge_x * point_y - edge_y * point_x;
        if cross > 0 {
            inside = false;
            break;
        }
    }

    inside
}

fn get_face_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0],
    ]
}

const SHADE_CHARS: [char; 8] = [' ', '.', ':', '-', '=', '+', '#', '@'];

fn render_cube(width: usize, height: usize, angle_x: f32, angle_y: f32, angle_z: f32) -> String {
    let mut buffer: Vec<(i32, i32, f32, char, Color)> = Vec::new();
    let fov = 40.0;
    let distance = 5.0;

    // Transform vertices
    let mut transformed: [[f32; 3]; 8] = [[0.0; 3]; 8];
    for (i, v) in CUBE_VERTICES.iter().enumerate() {
        let mut p = *v;
        p = rotate_x(p, angle_x);
        p = rotate_y(p, angle_y);
        p = rotate_z(p, angle_z);
        transformed[i] = p;
    }

    // Sort faces by depth and render back-to-front
    let mut face_depths: Vec<(usize, f32, [f32; 3])> = Vec::new();

    for (i, (indices, _)) in CUBE_FACES.iter().enumerate() {
        let v0 = transformed[indices[0]];
        let v1 = transformed[indices[1]];
        let v2 = transformed[indices[2]];

        // Calculate face center depth
        let center_z = (v0[2] + v1[2] + transformed[indices[2]][2] + transformed[indices[3]][2]) / 4.0;

        // Calculate normal for backface culling
        let normal = get_face_normal(v0, v1, v2);

        face_depths.push((i, center_z, normal));
    }

    // Sort back to front
    face_depths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Render faces
    for (face_idx, _, normal) in &face_depths {
        let (indices, color) = &CUBE_FACES[*face_idx];

        // Backface culling - skip faces pointing away
        if normal[2] < 0.0 { continue; }

        // Calculate shading based on normal
        let light_dir = [0.0, 0.0, 1.0];
        let dot = normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2];
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let intensity = if len > 0.0 { (dot / len).max(0.0) } else { 0.0 };

        let shade_idx = ((intensity * (SHADE_CHARS.len() - 1) as f32) as usize).min(SHADE_CHARS.len() - 1);
        let shade_char = SHADE_CHARS[shade_idx];

        let face_verts = [
            transformed[indices[0]],
            transformed[indices[1]],
            transformed[indices[2]],
            transformed[indices[3]],
        ];

        fill_face(&face_verts, width, height, fov, distance, &mut buffer, *color, shade_char);
    }

    // Draw edges on top
    for (i, j) in CUBE_EDGES.iter() {
        if let (Some(p1), Some(p2)) = (
            project(transformed[*i], width, height, fov, distance),
            project(transformed[*j], width, height, fov, distance),
        ) {
            let avg_z = (p1.2 + p2.2) / 2.0;
            draw_line(p1.0, p1.1, p2.0, p2.1, &mut buffer, avg_z - 0.1, Color::White);
        }
    }

    // Create z-buffer for proper depth
    let mut z_buffer: Vec<f32> = vec![f32::MAX; width * height];
    let mut char_buffer: Vec<(char, Color)> = vec![(' ', Color::Black); width * height];

    for (x, y, z, c, color) in buffer {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let idx = y as usize * width + x as usize;
            if z < z_buffer[idx] {
                z_buffer[idx] = z;
                char_buffer[idx] = (c, color);
            }
        }
    }

    // Build output string with colors
    let mut output = String::with_capacity(width * height * 20);
    let mut current_color: Option<Color> = None;

    for y in 0..height {
        for x in 0..width {
            let (c, color) = char_buffer[y * width + x];

            if c != ' ' {
                if current_color != Some(color) {
                    output.push_str(&format!("{}", SetForegroundColor(color)));
                    current_color = Some(color);
                }
                output.push(c);
            } else {
                if current_color.is_some() {
                    output.push_str(&format!("{}", ResetColor));
                    current_color = None;
                }
                output.push(' ');
            }
        }
    }

    if current_color.is_some() {
        output.push_str(&format!("{}", ResetColor));
    }

    output
}

fn main() -> std::io::Result<()> {
    let mut last_render_time = std::time::Instant::now();
    let start_time = std::time::Instant::now();

    loop {
        let (cols, lines) = usable_space()?;

        // Calculate rotation based on time
        let elapsed = start_time.elapsed().as_secs_f32();
        let angle_x = elapsed * 0.7;
        let angle_y = elapsed * 1.0;
        let angle_z = elapsed * 0.3;

        let screen = render_cube(cols as usize, lines as usize, angle_x, angle_y, angle_z);

        // One line that wraps, \r goes back to start
        print!("\r{}", screen);

        print!(
            " --- Temps écoulé depuis le dernier rendu : {:?} ms ---",
            last_render_time.elapsed().as_millis()
        );

        std::io::Write::flush(&mut std::io::stdout())?;
        last_render_time = std::time::Instant::now();
    }
}