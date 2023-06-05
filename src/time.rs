use std::time;

pub struct TimeContext {
    pub(crate) start_time: time::SystemTime,
    pub(crate) current_time: time::SystemTime,
}

impl Default for TimeContext {
    fn default() -> Self {
        let start_time = std::time::SystemTime::now();
        Self {
            start_time,
            current_time: start_time,
        }
    }
}

impl TimeContext {
    pub(crate) fn update_time(&mut self) -> f32 {
        let new_time = std::time::SystemTime::now();
        let dt = new_time
            .duration_since(self.current_time)
            .unwrap()
            .as_secs_f32();
        self.current_time = new_time;
        dt
    }

    pub(crate) fn time_since_start(&self) -> f32 {
        let new_time = std::time::SystemTime::now();
        new_time
            .duration_since(self.start_time)
            .unwrap()
            .as_secs_f32()
    }
}
