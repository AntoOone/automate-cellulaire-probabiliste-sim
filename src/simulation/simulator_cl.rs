use crate::simulation::SimulationDevice;

use ocl::{Buffer, Kernel, ProQue};

pub struct SimulatorCl {
    pro_queue: ProQue,
    kernel: Kernel,
    tab_cl: Buffer<u8>,
    background_buffer: Buffer<u8>,
    tab: Vec<u8>,
}

impl SimulationDevice for SimulatorCl {
    fn new(initial_state: Vec<u8>, k: u8, n: u32, d: u8) -> Self {
        let src = include_str!("simulation.cl");

        let dim: u32 = 1024;
        let pro_queue = ProQue::builder().src(src).dims(dim).build().unwrap();

        let background_buffer = Buffer::<u8>::builder()
            .queue(pro_queue.queue().clone())
            .len(initial_state.len())
            .build()
            .unwrap();

        let tab_cl = Buffer::<u8>::builder()
            .queue(pro_queue.queue().clone())
            .len(initial_state.len())
            .copy_host_slice(&initial_state)
            .build()
            .unwrap();

        let random_elems: Vec<u32> = (0..dim).map(|_| rand::random::<u32>()).collect();

        let random_elems_cl = Buffer::<u32>::builder()
            .queue(pro_queue.queue().clone())
            .len(dim as usize)
            .copy_host_slice(&random_elems)
            .build()
            .unwrap();

        let nb_cells = n.pow(d as u32) as usize;
        let q = nb_cells / (dim as usize);
        let r = nb_cells % (dim as usize);

        let kernel = pro_queue
            .kernel_builder("simulation_step")
            .arg(&tab_cl)
            .arg(&background_buffer)
            .arg(&random_elems_cl)
            .arg(&(0 as u32))
            .arg(k)
            .arg(n)
            .arg(d)
            .arg(dim as u32)
            .arg(q as u32)
            .arg(r as u32)
            .build()
            .unwrap();

        Self {
            pro_queue,
            kernel,
            tab_cl,
            background_buffer,
            tab: vec![0; initial_state.len()],
        }
    }

    fn simulate_step(&mut self, iteration: u32) {
        self.kernel.set_arg(3, iteration).unwrap();
        unsafe {
            self.kernel.enq().unwrap();
        }
        self.pro_queue.finish().unwrap();

        // On change échange des buffers
        std::mem::swap(&mut self.tab_cl, &mut self.background_buffer);
        self.kernel.set_arg(0, &self.tab_cl).unwrap();
        self.kernel.set_arg(1, &self.background_buffer).unwrap();
    }

    fn read_tab(&self) -> &Vec<u8> {
        &self.tab
    }

    fn prepare_reading(&mut self) {
        self.tab_cl.read(&mut self.tab).enq().unwrap();
    }
}
