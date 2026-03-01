pub struct SimulationData {
    pub n: u32,
    pub d: u8,
    pub k: u8,
}

impl SimulationData {
    pub fn new(n: u32, d: u8, k: u8) -> Self {
        Self { n, d, k }
    }
}
