use std::time;

pub struct TimeContext {
    pub current_time: time::SystemTime,
}

impl Default for TimeContext {
    fn default() -> Self {
        let current_time = std::time::SystemTime::now();
        Self { current_time }
    }
}

impl TimeContext {
    pub fn update_time(&mut self) -> f32 {
        let new_time = std::time::SystemTime::now();
        let time_since = self.time_since(new_time);
        self.current_time = new_time;
        time_since
    }

    fn time_since(&self, new_time: std::time::SystemTime) -> f32 {
        new_time
            .duration_since(self.current_time)
            .unwrap()
            .as_secs_f32()
    }
}
