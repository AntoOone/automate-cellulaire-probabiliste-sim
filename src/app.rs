mod renderer;

use std::{error::Error, time::Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::{
    app::renderer::clear_screen,
    simulation::{SimulationDevice, Simulator},
    simulation_save::SimulationSave,
};

pub enum DisplayMethod {
    DisplayIntervalle(u32),
    SimulationFrequency(f32),
}

pub struct App<S: SimulationDevice> {
    simulator: Simulator<S>,
    computing_start_time: Instant,

    display_method: DisplayMethod,
    last_rendering_time: Instant,
    cell_to_color: Vec<(u8, u8, u8)>,

    save_intervalle: u32,
    save_path: String,
    simulation_save: SimulationSave,

    last_rendering_iteration: u32,
    last_saving_iteration: u32,
    fist_iteration: bool,

    runing: bool,
}

impl<S: SimulationDevice> App<S> {
    pub fn new(
        simulator: Simulator<S>,
        display_method: DisplayMethod,
        cell_to_color: Vec<(u8, u8, u8)>,
        save_intervalle: u32,
        save_path: String,
        simulation_save: SimulationSave,
    ) -> Self {
        Self {
            simulator,
            computing_start_time: Instant::now(),

            display_method,
            last_rendering_time: Instant::now(),
            cell_to_color,

            save_intervalle,
            save_path,
            simulation_save,

            last_rendering_iteration: 0,
            last_saving_iteration: 0,
            fist_iteration: true,

            runing: false,
        }
    }
}

pub fn run<S: SimulationDevice + 'static>(mut app: App<S>) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Simulation")
        .build(&event_loop)?;
    let size = window.inner_size();
    let surface = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels = Pixels::new(size.width, size.height, surface)?;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width > 0 && size.height > 0 {
                    pixels.resize_surface(size.width, size.height).unwrap();
                    pixels.resize_buffer(size.width, size.height).unwrap();
                    // render_app(&mut app, &mut pixels);
                    clear_screen(pixels.frame_mut());
                    render_app(&mut app, &mut pixels);
                }
            }

            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                ..
            } => {
                if new_inner_size.width > 0 && new_inner_size.height > 0 {
                    pixels
                        .resize_surface(new_inner_size.width, new_inner_size.height)
                        .unwrap();
                    pixels
                        .resize_buffer(new_inner_size.width, new_inner_size.height)
                        .unwrap();
                    clear_screen(pixels.frame_mut());
                    render_app(&mut app, &mut pixels);
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                app.simulator.prepare_reading();
                let tab = app.simulator.read_tab();
                app.simulation_save.update(
                    tab,
                    Instant::now()
                        .duration_since(app.computing_start_time)
                        .as_secs_f32(),
                    app.simulator.iteration(),
                    true,
                );
                app.simulation_save.save(&app.save_path).unwrap();

                crossterm::terminal::disable_raw_mode().unwrap();
                control_flow.set_exit();
                return;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.scancode == 57 && input.state == ElementState::Pressed {
                    // Space pressed
                    app.runing = !app.runing;
                    match app.runing {
                        true => print!("RUNING\r\n"),
                        false => print!("PAUSE\r\n"),
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if app.runing {
                    if app.fist_iteration {
                        app.fist_iteration = false;
                        app.simulator.prepare_reading();
                        render_app(&mut app, &mut pixels);
                        save_app(&mut app);
                    } else {
                        save_and_render_app(&mut app, &mut pixels);
                        update_app(&mut app);
                    }
                }
            }
            _ => (),
        }
    });
}

fn render_app<S: SimulationDevice>(app: &mut App<S>, pixels: &mut Pixels) {
    let texture = pixels.texture();
    let width = texture.width();
    let height = texture.height();
    let mut frame = pixels.frame_mut();

    let tab = app.simulator.read_tab();
    renderer::render::<S>(
        &mut frame,
        width as usize,
        height as usize,
        tab,
        &app.cell_to_color,
        app.simulator.data.n,
        app.simulator.data.d,
    );
    pixels.render().unwrap();
}

fn save_app<S: SimulationDevice>(app: &mut App<S>) {
    let tab = app.simulator.read_tab();
    app.simulation_save.update(
        tab,
        Instant::now()
            .duration_since(app.computing_start_time)
            .as_secs_f32(),
        app.simulator.iteration(),
        false,
    );
}

fn save_and_render_app<S: SimulationDevice>(app: &mut App<S>, pixels: &mut Pixels) {
    let render = (app.last_rendering_iteration < app.simulator.iteration())
        && match app.display_method {
            DisplayMethod::DisplayIntervalle(display_intervalle) => {
                app.simulator.iteration() % display_intervalle == 0
            }
            DisplayMethod::SimulationFrequency(_) => true,
        };
    let save = app.simulator.iteration() % app.save_intervalle == 0
        && app.last_saving_iteration < app.simulator.iteration();
    if render || save {
        app.simulator.prepare_reading();
    }
    if render {
        app.last_saving_iteration = app.simulator.iteration();
        render_app(app, pixels);
    }
    if save {
        app.last_rendering_iteration = app.simulator.iteration();
        save_app(app);
    }
}

/// Met à jour l'application et la simulation
fn update_app<S: SimulationDevice>(app: &mut App<S>) {
    if let DisplayMethod::SimulationFrequency(frequency) = app.display_method {
        let now = Instant::now();
        if now.duration_since(app.last_rendering_time).as_secs_f32() >= 1f32 / frequency {
            app.last_rendering_time = now;
            app.simulator.simulate_step();
        }
    } else {
        app.simulator.simulate_step();
    }
}
