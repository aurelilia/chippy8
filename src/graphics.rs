use imgui::*;
use glium::{glutin, Surface};
use std::process;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{WinitPlatform, HiDpiMode};
use crate::chip8::Chip8;

pub struct System {
    events: glutin::EventsLoop,
    display: glium::Display,
    imgui: Context,
    platform: WinitPlatform,
    pub chip8: Chip8,
    renderer: Renderer,
}

pub fn draw(system: &mut System) {
    let gl_window = system.display.gl_window();
    let window = gl_window.window();

    let io = system.imgui.io_mut();
    system.platform
        .prepare_frame(io, &window)
        .expect("Failed to start frame");


    let mut ui = system.imgui.frame();
    super::draw_gui(&mut ui, &system.chip8);

    let mut target = system.display.draw();
    target.clear_color(0.05, 0.05, 0.05, 1.0);
    system.platform.prepare_render(&ui, &window);

    let ui_data = ui.render();
    system.renderer.render(&mut target, ui_data).unwrap();

    target.finish().unwrap();
}

pub fn input(system: &mut System) {
    let events = &mut system.events;
    let platform = &mut system.platform;
    let display = &mut system.display;
    let imgui = &mut system.imgui;
    let chip8 = &mut system.chip8;

    events.poll_events(|ev| {
        platform.handle_event(imgui.io_mut(), display.gl_window().window(), &ev);
        if let glutin::Event::WindowEvent { event, .. } = ev {
            crate::handle_input(event, chip8)
        }
    });
}

pub fn setup(chip8: Chip8) -> System {
    let mut events = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new()
        .with_title("chippy8")
        .with_dimensions(glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events).unwrap();

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
    }

    let renderer = Renderer::init(&mut imgui, &display).unwrap();

    System {
        events,
        display,
        imgui,
        platform,
        chip8,
        renderer
    }
}
