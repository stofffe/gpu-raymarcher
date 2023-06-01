use crate::{input::InputContext, render::RenderContext, time::TimeContext};

/// Holds all the neccesary state for running the engine
/// Should be sent with each command
pub struct Context {
    pub(crate) render: RenderContext,
    pub(crate) time: TimeContext,
    pub(crate) input: InputContext,
}
