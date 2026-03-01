mod simulation_data;
#[cfg(feature = "opencl")]
pub mod simulator_cl;
#[cfg(not(feature = "opencl"))]
pub mod simulator_cpu;

use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use serde::Deserialize;
use simulation_data::SimulationData;
use std::error::Error;

/// Appareils sur lesquelles la simulation va s'exécuter : CPU ou GPU avec OpenCL.
pub trait SimulationDevice {
    fn new(initial_state: Vec<u8>, k: u8, n: u32, d: u8) -> Self;

    /// Simule une étape sur le device en question.
    fn simulate_step(&mut self, iteration: u32);

    /// Prépare la simulation à ce qu'on puisse lire le tableau des cellules (donc à read_tab).
    /// C'est nécessaire car dans le cas d'OpenCL, le tableau n'existe pas dans la RAM avant qu'on en ait fait la copie.
    /// Ceci explique la nécessité d'avoir &mut et non & : on copie du device à l'host.
    fn prepare_reading(&mut self);

    /// Retourne une éférence vers le tableau des cellules.
    /// Il est nécessaire d'avoir fait un prepare_reading avant read_tab dans l'étape de la simulation.
    /// Il est cependant attendu d'avoir plusieurs read_tab après un prepare_reading.
    fn read_tab(&self) -> &Vec<u8>;
}

pub struct Simulator<S: SimulationDevice> {
    pub data: SimulationData,
    simulation_iteration: u32,
    sim_device: S,
}

/// Structure du json d'entrée si l'on veut utiliser l'affichage
#[derive(Deserialize)]
struct SimulationInputColored {
    d: u8,
    tab: Vec<u8>,
    cell_to_color: Vec<Vec<u8>>,
}

/// Structure du json d'entrée sans l'affichage
#[derive(Deserialize)]
struct SimulationInput {
    d: u8,
    tab: Vec<u8>,
}

impl<S: SimulationDevice> Simulator<S> {
    fn new(initial_tab: Vec<u8>, n: u32, d: u8) -> Result<Self, Box<dyn Error>> {
        let k = (*initial_tab)
            .iter()
            .max()
            .ok_or("Impossible de trouver le max du tableau initial".to_string())?
            .checked_add(1)
            .ok_or("Il ne peut pas y avoir plus de 256 catégories différentes".to_string())?;
        if initial_tab.len() == 0 {
            return Err("Le tableau initial est vide".into());
        }
        if n.pow(d as u32) as usize != initial_tab.len() {
            return Err("Les dimensions ne correspondent pas à la taille du tableau".into());
        }
        Ok(Self {
            simulation_iteration: 0,
            data: SimulationData::new(n, d, k),
            sim_device: S::new(initial_tab, k, n, d),
        })
    }

    /// Depuis une image : retourne le simulateur et le vecteur des couleurs associées aux catégories des cellules
    pub fn from_image(img: DynamicImage) -> Result<(Self, Vec<(u8, u8, u8)>), Box<dyn Error>> {
        let (n, nb_cells) = {
            let (w, h) = (img.width(), img.height());
            if w != h {
                return Err("La hauteur et la largeur de l'image doivent être égales".into());
            }
            (w, (w * h) as usize)
        };
        let pixels = img.pixels();
        let mut tab: Vec<u8> = vec![0; nb_cells];
        let mut cell_to_color: Vec<(u8, u8, u8)> = Vec::new();
        let mut k: u8 = 0;

        for (x, y, rgba) in pixels {
            let var_name = rgba.to_rgb().0;
            let [r, g, b] = var_name;
            let color = (r, g, b);
            let i = (n * y + x) as usize;

            let cell = if let Some(c) = cell_to_color.iter().position(|c| *c == color) {
                c as u8
            } else {
                let m = k;
                k = k.checked_add(1).ok_or(
                    "Il ne peux pas y avoir plus de 256 couleurs de cellules différentes"
                        .to_string(),
                )?;
                cell_to_color.push(color);
                m
            };
            tab[i] = cell;
        }
        Ok((Self::new(tab, n, 2)?, cell_to_color))
    }

    /// Depuis un fichier json : retourne le simulateur et le potentiel vecteur des couleurs associées aux catégories des cellules
    pub fn from_json(json: &str) -> Result<(Self, Option<Vec<(u8, u8, u8)>>), Box<dyn Error>> {
        let (d, tab, cell_to_color) =
            if let Ok(sim) = serde_json::from_str::<SimulationInputColored>(json) {
                (sim.d, sim.tab, Some(sim.cell_to_color))
            } else {
                let sim = serde_json::from_str::<SimulationInput>(json)?;
                (sim.d, sim.tab, None)
            };

        // On détermine le n associé au tableau, si il est compatible
        let n = (tab.len() as f32).powf(1.0 / (d as f32)) as u32;
        if n.pow(d as u32) as usize != tab.len() {
            return Err("Mauvais dimensionnement du tableau".into());
        }

        Ok((
            Self::new(tab, n, d)?,
            cell_to_color.map(|t| t.iter().map(|v| (v[0], v[1], v[2])).collect()),
        ))
    }

    fn try_open_image(path: String) -> Result<DynamicImage, Box<dyn Error>> {
        Ok(ImageReader::open(path)?.decode()?)
    }

    /// Depuis un fichier : retourne le simulateur et le potentiel vecteur des couleurs associées aux catégories des cellules
    pub fn from_file(path: String) -> Result<(Self, Option<Vec<(u8, u8, u8)>>), Box<dyn Error>> {
        if let Ok(img) = Self::try_open_image(path.clone()) {
            let (v, c) = Self::from_image(img)?;
            Ok((v, Some(c)))
        } else {
            let json = std::fs::read_to_string(path)?;
            Ok(Self::from_json(&json)?)
        }
    }

    pub fn iteration(&self) -> u32 {
        self.simulation_iteration
    }

    pub fn simulate_step(&mut self) {
        self.sim_device.simulate_step(self.simulation_iteration);
        self.simulation_iteration += 1;
    }

    /// Prépare la lecture de tab dans le device (voir SimulationDevice)
    pub fn prepare_reading(&mut self) {
        self.sim_device.prepare_reading();
    }

    /// Lis tab dans le device (voir SimulationDevice)
    pub fn read_tab(&self) -> &Vec<u8> {
        self.sim_device.read_tab()
    }
}
