use std::error::Error;
use std::fs::File;
use std::process::exit;
use clap::Parser;
use pixels::{Pixels, SurfaceTexture};
use clover::{Clover, Object, Program, State};
use clover_std::clover_std_inject_to;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    dpi::LogicalSize,
    window::WindowBuilder,
};
use legend_engine::engine::graphics::Graphics;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 200;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// window scale
    #[clap(short, long, value_parser = clap::value_parser!(u32).range(1...10), default_value_t = 2)]
    scale: u32,

    /// folder which contain the original Legend game install path or CD
    #[clap(value_parser)]
    data_path: String,
}

fn init_script() -> Result<(State, Object, Object), Box<dyn Error>> {
    let clover = Clover::new();

    let program = clover.compile_file("./scripts/main.luck")?;

    let mut state: State = program.into();
    clover_std_inject_to(&mut state);

    let game = state.execute()?;
    let update_function = state.get_object_property_by_name(game.clone(), "update")?;
    let render_function = state.get_object_property_by_name(game.clone(), "render")?;

    Ok((state, update_function, render_function))
}

fn init_engine() -> Result<(Graphics), Box<dyn Error>> {
    Ok((Graphics::new(WIDTH, HEIGHT)?))
}

fn run_frame(graphics: &mut Graphics, state: &mut State, update_function: &Object, render_function: &Object, pixels: &mut Pixels) -> Result<(), Box<dyn Error>> {
    let update_result = state.execute_by_object(update_function.clone(), &[ Object::Float(0.0) ])?;
    let render_result = state.execute_by_object(render_function.clone(), &[ Object::Float(0.0) ])?;

    let frame_buffer = pixels.get_frame();

    graphics.render_to(frame_buffer)?;

    pixels.render()?;


    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let (mut graphics) = init_engine()?;
    let (mut state, update_function, render_function) = init_script()?;

    let event_loop = EventLoop::new();
    let window = {
        let scale = args.scale;

        let size = LogicalSize::new(WIDTH * scale, HEIGHT * scale);
        WindowBuilder::new()
            .with_title("Legend Clover")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop).unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }

        if run_frame(&mut graphics, &mut state, &update_function, &render_function, &mut pixels).is_err() {
            *control_flow = ControlFlow::Exit;
        }
    });
}