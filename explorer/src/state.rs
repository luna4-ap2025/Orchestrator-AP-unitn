use common_game::components::resource::{BasicResource, ComplexResource};

pub struct ExplorerState {
    pub current_planet: Option<u32>,
    pub basic_resources: Vec<BasicResource>,
    pub complex_resources: Vec<ComplexResource>,
}

impl ExplorerState {
    pub fn new() -> Self {
        Self {
            current_planet: None,
            basic_resources: Vec::new(),
            complex_resources: Vec::new(),
        }
    }
}
