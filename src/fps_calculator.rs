
use std::time::Instant;

pub struct FPScalculator {
    frames: usize,
    start: Option<Instant>,
}

impl FPScalculator {
    pub fn new() -> Self {
        Self {
            frames: 0,
            start: None
        }
    }

    pub fn count_one_frame(&mut self) {
        self.frames += 1;
    }

    pub fn fps(&mut self) -> f32 {
        let fps = match self.start {
            Some(start) => {
                let elapsed = start.elapsed().as_secs_f32();

                self.frames as f32 / elapsed
            }
            None => 0.0
        };
        self.frames = 0;
        self.start = Some(Instant::now());

        fps
    }

}