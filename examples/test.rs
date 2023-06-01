use gpu_raymarcher::{Callbacks, Context};

struct App {}

impl Callbacks for App {
    fn init(&self, ctx: &mut Context) {}

    fn update(&mut self, _ctx: &mut Context, _dt: f32) -> bool {
        false
    }
}

fn main() {
    let app = App {};
    gpu_raymarcher::run(app);
}
