use glam::Vec2;
use sdl3::{event::Event, keyboard::Keycode};
use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    color::Color,
    rect::{Rect, RectParams, RectPipelineManager},
    window::UIWindow,
};

mod buffer;
mod color;
mod pipeline;
mod rect;
mod window;

fn main() -> Result<(), Box<dyn Error>> {
    let context = sdl3::init()?;
    let video_subsystem = context.video()?;

    let mut window = UIWindow::new(&video_subsystem, "Rust SDL3", 800, 600)?;
    let rect_pipeline = Rc::new(RefCell::new(RectPipelineManager::new()));
    window.add_pipeline(rect_pipeline.clone())?;

    let mut event_pump = context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        rect_pipeline.borrow_mut().draw(RectParams::new(
            Rect::new(10.0, 20.0, 200.0, 300.0),
            Color::RGBA(0.5, 0.6, 0.8, 0.5),
            [5.0, 10.0, 20.0, 40.0],
            10.0,
            Color::WHITE,
            Color::RGB(0.5, 0.5, 0.5),
            Vec2::new(0.0, 0.0),
            10.0,
        ));

        rect_pipeline.borrow_mut().draw(RectParams::new(
            Rect::new(50.0, 80.0, 100.0, 500.0),
            Color::RGBA(0.8, 0.6, 0.5, 0.5),
            [5.0, 10.0, 20.0, 40.0],
            5.0,
            Color::RGBA(0.4, 0.3, 0.2, 0.8),
            Color::RGB(0.5, 0.5, 0.5),
            Vec2::new(0.0, 0.0),
            10.0,
        ));

        window.update()?;
    }

    Ok(())
}
