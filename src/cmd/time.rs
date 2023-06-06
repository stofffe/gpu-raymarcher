use std::time;

use crate::Context;

pub fn time_since_start(ctx: &Context) -> f32 {
    ctx.time.time_since_start()
}

pub fn current_time(ctx: &Context) -> time::SystemTime {
    ctx.time.current_time
}
