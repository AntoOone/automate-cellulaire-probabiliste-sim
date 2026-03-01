use crate::simulation::SimulationDevice;

pub fn clear_screen(frame: &mut [u8]) {
    frame.fill(0);
}

pub fn render<S: SimulationDevice>(
    frame: &mut [u8],
    screen_width: usize,
    screen_height: usize,

    tab: &Vec<u8>,
    cell_to_color: &Vec<(u8, u8, u8)>,
    n: u32,
    d: u8,
) {
    match d {
        1 => render_d1::<S>(frame, screen_width, screen_height, tab, cell_to_color, n),
        2 => render_d2::<S>(frame, screen_width, screen_height, tab, cell_to_color, n),
        3 => panic!("Le cas d = 3 n'a pas d'option d'affichage"),
        _ => panic!("Le cas d > 3 n'a pas d'option d'affichage"),
    }
}

pub fn render_d1<S: SimulationDevice>(
    frame: &mut [u8],
    screen_width: usize,
    screen_height: usize,

    tab: &Vec<u8>,
    cell_to_color: &Vec<(u8, u8, u8)>,
    n: u32,
) {
    let s = n as usize;

    let resolution_canvas = n as f32;
    let resolution_screen = (screen_width as f32) / (screen_height as f32);

    let mut canvas_width = if resolution_screen > resolution_canvas {
        // On ajoute un padding horizontal
        ((screen_width as f32) / resolution_canvas) as usize
    } else {
        // On ajoute un padding vertical
        screen_width
    };

    let cell_size = std::cmp::max(canvas_width / s, 1);
    let cell_step = s / canvas_width + 1;
    print!("step={}   size={}\r\n", cell_step, cell_size);

    canvas_width = cell_size * s / cell_step;
    let canvas_height = cell_size / cell_step;

    let canvas_x_offset = (screen_width - canvas_width + (canvas_width % cell_size)) / 2;
    let canvas_y_offset = (screen_height - canvas_height + cell_size) / 2;

    // On dessine les cellules de la simulation
    for y in 0..cell_size {
        for cell_x in (0..s).step_by(cell_step) {
            let v = tab[cell_x] as usize;
            if v >= cell_to_color.len() {
                panic!("La valeur de la cellule n'a pas de couleur associée: index out of range");
            }
            let (r, g, b) = cell_to_color[v];
            for x in 0..cell_size {
                let (pixel_x, pixel_y) = (
                    canvas_x_offset + cell_x * cell_size / cell_step + x,
                    canvas_y_offset + y,
                );
                let index = pixel_x + screen_width * pixel_y;
                frame[4 * index] = r;
                frame[4 * index + 1] = g;
                frame[4 * index + 2] = b;
                frame[4 * index + 3] = 255;
            }
        }
    }
}

pub fn render_d2<S: SimulationDevice>(
    frame: &mut [u8],
    screen_width: usize,
    screen_height: usize,

    tab: &Vec<u8>,
    cell_to_color: &Vec<(u8, u8, u8)>,
    n: u32,
) {
    let s = n as usize;

    let resolution_canvas = 1f32;
    let resolution_screen = (screen_width as f32) / (screen_height as f32);

    let mut canvas_size = if resolution_screen > resolution_canvas {
        // On ajoute un padding horizontal
        screen_height
    } else {
        // On ajoute un padding vertical
        ((screen_width as f32) / resolution_canvas) as usize
    };

    let cell_size = std::cmp::max(canvas_size / s, 1);
    let cell_step = s / canvas_size + 1;

    canvas_size = cell_size * s / cell_step;

    let canvas_x_offset = (screen_width - canvas_size + (canvas_size % cell_size)) / 2;
    let canvas_y_offset = (screen_height - canvas_size + (canvas_size % cell_size)) / 2;

    // On dessine les cellules de la simulation
    for cell_y in (0..s).step_by(cell_step) {
        for y in 0..cell_size {
            for cell_x in (0..s).step_by(cell_step) {
                let v = tab[s * cell_y + cell_x] as usize;
                if v >= cell_to_color.len() {
                    panic!(
                        "La valeur de la cellule n'a pas de couleur associée: index out of range"
                    );
                }
                let (r, g, b) = cell_to_color[v];
                for x in 0..cell_size {
                    let (pixel_x, pixel_y) = (
                        canvas_x_offset + cell_x * cell_size / cell_step + x,
                        canvas_y_offset + cell_y * cell_size / cell_step + y,
                    );
                    let index = pixel_x + screen_width * pixel_y;
                    frame[4 * index] = r;
                    frame[4 * index + 1] = g;
                    frame[4 * index + 2] = b;
                    frame[4 * index + 3] = 255;
                }
            }
        }
    }
}
