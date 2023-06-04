use gpu_raymarcher::{Callbacks, Context};

struct App {}

impl Callbacks for App {
    fn init(&self, ctx: &mut Context) {}

    fn update(&mut self, _ctx: &mut Context, dt: f32) -> bool {
        println!("{dt}");
        false
    }
}

fn main() {
    let app = App {};
    gpu_raymarcher::run(app);
}
