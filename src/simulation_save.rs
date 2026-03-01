use serde::Serialize;
use std::error::Error;

/// Structure de sauvegarde de la simulation
#[derive(Serialize)]
pub struct SimulationSave {
    d: u8,
    n: u32,
    k: u8,

    nb_iterations: u32,
    computing_time: f32,
    iterations_per_second: f32,
    save_interval: u32,

    // champs statistiques des états de la simulation à chaque update
    count: Vec<Vec<u32>>, // count[k][i] contient le nombre de cellules de l'opinion k à l'itération i
    frequency: Vec<Vec<f32>>, // frequency[k][i] contient la fréquence de l'opinion k par rapport aux autres, à l'itération i
    entropy: Vec<f32>,        // entropy[i] contient l'entropie de la simulation à l'itération i
}

impl SimulationSave {
    pub fn new(d: u8, n: u32, k: u8, iteration_interval: u32) -> Self {
        Self {
            n,
            d,
            k,

            nb_iterations: 0,
            computing_time: 0f32,
            iterations_per_second: 0f32,
            save_interval: iteration_interval,

            count: vec![Vec::new(); k as usize],
            frequency: vec![Vec::new(); k as usize],
            entropy: Vec::new(),
        }
    }

    /// Sauvegarde la simulation au format json dans le fichier indiqué
    pub fn save(&self, save_path: &String) -> Result<(), Box<dyn Error>> {
        print!("Sauvegarde des données de la simulation\r\n");
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(save_path, json)?;
        Ok(())
    }

    /// Ajoute les données de l'itération de la simulation à la structure de sauvegarde
    pub fn update(
        &mut self,
        tab: &Vec<u8>,
        computing_time: f32,
        iteration: u32,
        last_update: bool,
    ) {
        let mut cat_count = vec![0; self.k as usize];
        for c in tab {
            cat_count[*c as usize] += 1;
        }
        let total_count: u32 = cat_count.iter().sum();
        let mut e = 0f32;
        for i in 0..self.k {
            let c = cat_count[i as usize];
            let f = (cat_count[i as usize] as f32) / (total_count as f32);
            e -= if f == 0f32 { 0f32 } else { f * f.ln() };
            self.count[i as usize].push(c);
            self.frequency[i as usize].push(f);
        }
        self.entropy.push(e);

        if last_update {
            self.nb_iterations = iteration;
            self.computing_time = computing_time;
            self.iterations_per_second = (iteration as f32) / computing_time;
        }
        print!("iteration : {}      entropy : {}\r\n", iteration, e);
    }
}
