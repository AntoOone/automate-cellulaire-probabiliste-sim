use crossterm::event::{Event, KeyCode, poll, read};
use maths_appli_simulation::{
    app::{self, App, DisplayMethod},
    simulation::{SimulationDevice, Simulator},
    simulation_save::SimulationSave,
};
use std::{error::Error, path::Path, time::Instant};

#[cfg(feature = "opencl")]
use maths_appli_simulation::simulation::simulator_cl::SimulatorCl;
#[cfg(not(feature = "opencl"))]
use maths_appli_simulation::simulation::simulator_cpu::SimulatorCPU;

/// Options d'affichage de la simulation
///
/// App : interface graphique qui fonctionne pour d = 1 ou 2
/// Hide : qui affiche des données dans le termial
enum Display {
    App { display_method: DisplayMethod },
    Hide { max_iteration: Option<u32> },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // DEFAULT ARGS
    let mut path = "simulation_step/config300x300.png".to_string();
    let mut display = Display::App {
        display_method: DisplayMethod::SimulationFrequency(60f32),
    };
    let mut save_interval = 20;
    let mut save_file_path = "simulation_save/sim.json".to_string();

    handle_args(
        &mut path,
        &mut display,
        &mut save_interval,
        &mut save_file_path,
    )?;

    #[cfg(feature = "opencl")]
    run_simulation::<SimulatorCl>(path, display, save_interval, save_file_path)?;

    #[cfg(not(feature = "opencl"))]
    run_simulation::<SimulatorCPU>(path, display, save_interval, save_file_path)?;

    Ok(())
}

// Analyse les arguments en entrée pour l'exécution de la simulation
fn handle_args(
    path: &mut String,
    display: &mut Display,
    save_interval: &mut u32,
    save_file_path: &mut String,
) -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    args.next();

    // FICHIER DE CONFIGURATION
    if let Some(a) = args.next() {
        let p = Path::new(&a);
        if p.exists() {
            *path = a;
        } else {
            return Err(format!(
                "Mauvais arguments. Le fichier situé à l'adresse {} n'existe pas",
                p.to_str()
                    .ok_or("Impossible de lire l'adresse donnée".to_string())?
            )
            .into());
        }
    }

    // OPTIONS D'AFFICHAGE
    if let Some(a) = args.next() {
        *display = match a.as_str() {
            "display" => Display::App {
                display_method: {
                    let word = args
                        .next()
                        .ok_or("Vous devez entrer l'option d'affichage : iteration ou frequency")?;
                    let nb_arg = args
                        .next()
                        .ok_or("Un nombre après l'option d'affichage est attendu")?;

                    match word.as_str() {
                        "iteration" => DisplayMethod::DisplayIntervalle(
                            nb_arg
                                .parse()
                                .map_err(|_| "Mauvais argument : un entier u32 est attendu")?,
                        ),
                        "frequency" => DisplayMethod::SimulationFrequency(
                            nb_arg
                                .parse()
                                .map_err(|_| "Mauvais argument : un flotant f32 est attendu")?,
                        ),
                        _ => return Err("L'option d'affichage est invalide".into()),
                    }
                },
            },
            "hide" => Display::Hide {
                max_iteration: match args
                    .next()
                    .ok_or("Vous devez entrer le nombre d'itérations max (ou int)")?
                    .as_str()
                {
                    "inf" => None,
                    x => Some(
                        x.parse()
                            .map_err(|_| "Mauvais argument : inf ou un entier u32 est attendu")?,
                    ),
                },
            },
            _ => return Err("Mauvais arguments. Attendus : display ou hide".into()),
        };
    }

    // OPTIONS DE SAUVEGARDE
    if let Some(a) = args.next() {
        *save_file_path = a;
    }

    if let Some(a) = args.next() {
        *save_interval = a
            .parse()
            .map_err(|_| "Mauvais argument : Un entier u32 est attendu")?;
    }

    Ok(())
}

fn is_space_pressed() -> std::io::Result<bool> {
    if poll(std::time::Duration::from_millis(0))? {
        {
            if let Event::Key(event) = read()? {
                if event.code == KeyCode::Char(' ') {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// Lance la simulation (soit CPU soit GPU avec opencl)
fn run_simulation<S: SimulationDevice + 'static>(
    path: String,
    display: Display,
    save_interval: u32,
    save_file_path: String,
) -> Result<(), Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let (mut sim, cell_to_color) = Simulator::<S>::from_file(path)?;
    sim.prepare_reading();
    let mut sim_save = SimulationSave::new(sim.data.d, sim.data.n, sim.data.k, save_interval);
    match display {
        Display::App { display_method } => {
            if sim.data.d == 3 {
                return Err(
                    "La simulation ne prend en charge que l'affichage des dimensions d = 1 ou 2"
                        .into(),
                );
            }
            let app = App::new(
                sim,
                display_method,
                cell_to_color.ok_or("Pour avoir un affichage, vous devez spécifier les couleurs associées aux opinions des électeurs")?,
                save_interval,
                save_file_path,
                sim_save,
            );
            app::run(app).map_err(|e| {
                format!(
                    "Une erreur est survenue lors de l'initialisation de l'application d'affichage : {}",
                    e
                )
            })?;
        }
        Display::Hide { max_iteration } => {
            let computing_start_time = Instant::now(); // Pour la sauvegarde finale
            let mut i: u32 = 0;
            loop {
                i += 1;
                if sim.iteration() % save_interval == 0 {
                    // On sauvegarde l'état de la simulation
                    sim.prepare_reading();
                    let tab = sim.read_tab();
                    sim_save.update(
                        tab,
                        Instant::now()
                            .duration_since(computing_start_time)
                            .as_secs_f32(),
                        sim.iteration(),
                        false,
                    );
                }
                sim.simulate_step();
                if let Some(m) = max_iteration
                {
                    if sim.iteration() >= m{
                        break;
                    }
                }
                if i % 1000 == 0 {
                    if is_space_pressed()? {
                        break;
                    }
                }
            }

            // On sauvegarde les données de la dernière itération et de la simulation
            let tab = sim.read_tab();
            sim_save.update(
                tab,
                Instant::now()
                    .duration_since(computing_start_time)
                    .as_secs_f32(),
                sim.iteration(),
                true,
            );

            sim_save.save(&save_file_path).unwrap();
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
