use sdl3::{
    gpu::{Device, GraphicsPipeline},
    video::Window,
};

struct UIWindow {
    window: Window,
    device: Device,
    pipelines: Vec<GraphicsPipeline>,
}
