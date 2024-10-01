pub trait Stage {
    fn execute(&self) {}
}

pub struct Pipeline {
    stages: Vec<Box<dyn Stage>>,
}

impl Pipeline {
    // main loop of the pipeline, where each stage is executed
    // in order they are added.
    pub fn run(&self) {
        loop {}
    }
}
