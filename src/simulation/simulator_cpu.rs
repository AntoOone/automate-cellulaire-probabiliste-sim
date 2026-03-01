use rand::Rng;

use crate::simulation::SimulationDevice;

pub struct SimulatorCPU {
    pub tab: Vec<u8>,
    background_buffer: Option<Vec<u8>>,
    neighbours_coords: Vec<Vec<u8>>, // 0 : 0  ;  1 : +1  ;  2: -1
    rng: rand::rngs::ThreadRng,
    n: u32,
    d: u8,
}

impl SimulatorCPU {
    fn sum_pos(dir: &Vec<u8>, pos: Vec<u32>, n: u32) -> Vec<u32> {
        dir.iter()
            .zip(pos.iter())
            .map(|(d, x)| match (*d, *x) {
                (0, _) => *x,
                (1, x) if x == (n - 1) => 0,
                (2, 0) => n - 1,
                (1, _) => *x + 1,
                (2, _) => *x - 1,
                (_, _) => panic!("Erreur d'implémentation de la simulation CPU. Le vecteur de direction ne doit contenir que 0, 1 ou 2"),
            })
            .collect::<Vec<_>>()
    }

    fn simulate_step_d1(&mut self, _iteration: u32) {
        for x in 0..self.n {
            let random_index = self.rng.random_range(0..self.neighbours_coords.len());
            let rand_dir = &self.neighbours_coords[random_index];

            // ----- Code à changer selon d -----
            let index = x as usize;

            let next_pos = SimulatorCPU::sum_pos(&rand_dir, vec![x], self.n);
            let x_n = next_pos[0];
            let index_n = x_n as usize;
            // ----------------------------------

            let opinion = self.tab[index_n];
            self.background_buffer.as_mut().unwrap()[index] = opinion;
        }
    }

    fn simulate_step_d2(&mut self, _iteration: u32) {
        for y in 0..self.n {
            for x in 0..self.n {
                let random_index = self.rng.random_range(0..self.neighbours_coords.len());
                let rand_dir = &self.neighbours_coords[random_index];

                // ----- Code à changer selon d -----
                let index = (self.n * y + x) as usize;

                let next_pos = SimulatorCPU::sum_pos(&rand_dir, vec![x, y], self.n);
                let (x_n, y_n) = (next_pos[0], next_pos[1]);
                let index_n = (self.n * y_n + x_n) as usize;
                // ----------------------------------

                let opinion = self.tab[index_n];
                self.background_buffer.as_mut().unwrap()[index] = opinion;
            }
        }
    }
    fn simulate_step_d3(&mut self, _iteration: u32) {
        for z in 0..self.n {
            for y in 0..self.n {
                for x in 0..self.n {
                    let random_index = self.rng.random_range(0..self.neighbours_coords.len());
                    let rand_dir = &self.neighbours_coords[random_index];

                    // ----- Code à changer selon d -----
                    let index = (self.n * self.n * z + self.n * y + x) as usize;

                    let next_pos = SimulatorCPU::sum_pos(&rand_dir, vec![x, y, z], self.n);
                    let (x_n, y_n, z_n) = (next_pos[0], next_pos[1], next_pos[2]);
                    let index_n = (self.n * self.n * z_n + self.n * y_n + x_n) as usize;
                    // ----------------------------------

                    let opinion = self.tab[index_n];
                    self.background_buffer.as_mut().unwrap()[index] = opinion;
                }
            }
        }
    }
}

impl SimulationDevice for SimulatorCPU {
    fn new(initial_state: Vec<u8>, _k: u8, n: u32, d: u8) -> Self {
        Self {
            tab: initial_state,
            background_buffer: Some(vec![0; n.pow(d as u32) as usize]),
            neighbours_coords: (0..(2 * d))
                .map(|i| {
                    let mut v = vec![0; d as usize];
                    v[(i / 2) as usize] = 1 + i % 2;
                    v
                })
                .collect::<Vec<Vec<_>>>(),
            rng: rand::rng(),
            n,
            d,
        }
    }

    fn simulate_step(&mut self, iteration: u32) {
        match self.d {
            1 => self.simulate_step_d1(iteration),
            2 => self.simulate_step_d2(iteration),
            3 => self.simulate_step_d3(iteration),
            _ => panic!("Le cas d > 3 n'a pas été traité par la simulation CPU"),
        }

        std::mem::swap(&mut self.tab, &mut self.background_buffer.as_mut().unwrap());
    }

    fn read_tab(&self) -> &Vec<u8> {
        &self.tab
    }

    fn prepare_reading(&mut self) {}
}
