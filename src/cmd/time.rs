use std::time;

use crate::Context;

/// Returns the time since the start of the application
pub fn time_since_start(ctx: &Context) -> f32 {
    ctx.time.time_since_start()
}

/// Returns the current time at the start of the current frame
pub fn current_time(ctx: &Context) -> time::SystemTime {
    ctx.time.current_time
}
