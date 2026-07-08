const REFRESH_INTERVAL: f32 = 0.25;

#[derive(Default)]
pub(crate) struct FrameStats {
    vertices: u32,
    indices: u32,

    frame_count: u32,
    accumulated_time: f32,
    displayed_frame_time: f32,
}

impl FrameStats {
    pub(crate) fn set_time(&mut self, dt: f32) {
        self.accumulated_time += dt;
        self.frame_count += 1;

        if self.accumulated_time >= REFRESH_INTERVAL {
            self.displayed_frame_time = self.accumulated_time / self.frame_count as f32;
            self.accumulated_time = 0.0;
            self.frame_count = 0;
        }
    }

    fn frame_rate(&self) -> f32 {
        // Avoid dividing by zero when value is uninitialized
        if self.displayed_frame_time == 0.0 {
            0.0
        } else {
            1.0 / self.displayed_frame_time
        }
    }

    pub(crate) fn set_model(&mut self, vertices: u32, indices: u32) {
        self.vertices = vertices;
        self.indices = indices;
    }

    pub(crate) fn time(&self) -> String {
        format!(
            "Frame time: {:>5.2} ms  ({:>3.0} FPS)",
            1000.0 * self.displayed_frame_time,
            self.frame_rate()
        )
    }

    pub(crate) fn model(&self) -> String {
        format!("Vertices: {:>7}    Triangles: {:>7}", self.vertices, self.indices / 3,)
    }
}
