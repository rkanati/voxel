
mod array3d;
mod block;
mod chunk;
mod chunk_cache;
mod chunk_maker;
mod chunk_source;
mod chunk_store;
mod game;
mod gl;
mod halton;
mod math;
mod mesher;
mod shader;
mod stage;
mod texture;

use {
    crate::{
        gl::types::*,
        math::*,
    },
    std::{
        error::Error,
        ffi,
        ptr,
        time::{Duration, Instant},
        slice,
        str,
    },
    glutin::{
        event,
        event_loop::{ControlFlow, EventLoop},
        platform::unix::EventLoopWindowTargetExtUnix,
    },
};

type Event<'w> = event::Event<'w, ()>;

extern "system" fn on_gl_debug(
    _source:   GLenum,
    _type:     GLenum,
    _id:       GLuint,
    _severity: GLenum,
    length:    GLsizei,
    message:   *const GLchar,
    _user:     *mut ffi::c_void)
{
    let msg_slice = unsafe {
        slice::from_raw_parts(message as *const u8, length as usize)
    };

    eprintln!(
        "GL debug: {}",
        str::from_utf8(msg_slice)
            .unwrap_or("<error parsing debug message>")
    );
}

const FRAME_RATE: u64 = 60;
const TICK_RATIO: u64 = 1;

const TICK_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / (FRAME_RATE * TICK_RATIO));

type Context = glutin::WindowedContext<glutin::PossiblyCurrent>;

struct App {
    ctx:                    Context,
    ticks_since_prev_frame: u32,
    screen_dims:            V2,
    focused:                bool,
    inputs:                 game::Inputs,
    game:                   game::Game,
}

impl App {
    fn new(ctx: Context) -> Result<App, Box<dyn Error>> {
        let app = App {
            ctx,
            ticks_since_prev_frame: 0,
            screen_dims: V2::new(500., 500.),
            focused: false,
            inputs: game::Inputs::new(),
            game: game::Game::new()?,
        };

        Ok(app)
    }

    fn handle_event<'w> (&mut self, event: &Event) {
        use event::{Event::*, WindowEvent::*, DeviceEvent::*};
        match event {
            WindowEvent { event, .. } => match event {
                Resized(new_size) => {
                    let phys = new_size;//.to_physical(1.0);
                    self.ctx.resize(*phys);
                    let (w, h): (f64, f64) = (*phys).into();
                    self.screen_dims = V2::new(w as f32, h as f32);
                }

                Focused(focused) => {
                    self.focused = *focused;
                    let win = self.ctx.window();
                    win.set_cursor_grab(*focused).unwrap();
                    win.set_cursor_visible(!*focused);
                    eprintln!("focus: {}", self.focused);
                }

                KeyboardInput {
                    input: event::KeyboardInput { state, virtual_keycode: Some(vk), ..  },
                    ..
                } => {
                    use event::VirtualKeyCode as VK;
                    let down = *state == event::ElementState::Pressed;
                    match vk {
                        VK::W        => self.inputs.fore  = down,
                        VK::A        => self.inputs.left  = down,
                        VK::S        => self.inputs.back  = down,
                        VK::D        => self.inputs.right = down,
                        VK::Space    => self.inputs.up    = down,
                        VK::LControl => self.inputs.down  = down,
                        VK::LShift   => self.inputs.fast  = down,
                        VK::Z        => self.inputs.zoom  = down,
                        _ => { }
                    }
                }

                MouseInput { state, button, .. } => {
                    let down = *state == event::ElementState::Pressed;
                    use event::MouseButton::*;
                    match button {
                        Left  => self.inputs.smash = down,
                        Right => self.inputs.build = down,
                        _     => { }
                    }
                }

                _ => { }
            }

            DeviceEvent { event, .. } => match event {
                MouseMotion { delta: (x, y) } => {
                    self.inputs.cam_delta += V2::new(*x as f32, *y as f32);
                }

                _ => { }
            }

            _ => { }
        }
    }

    fn tick(&mut self) {
        let inputs = if self.focused {
            self.inputs.take()
        }
        else {
            game::Inputs::new()
        };

        self.game.tick(&inputs, TICK_INTERVAL.as_secs_f32());

        self.ticks_since_prev_frame += 1;
        if self.ticks_since_prev_frame == TICK_RATIO as u32 {
            self.ticks_since_prev_frame = 0;
            self.game.draw(self.screen_dims);
            self.ctx.swap_buffers().unwrap();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_q = EventLoop::new();
    eprintln!(
        "Running on {}",
        if event_q.is_wayland() { "Wayland" } else { "X11" }
    );

    let ctx = {
        let win_builder = glutin::window::WindowBuilder::new()
            .with_title("voxels");

        glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Latest)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl_debug_flag(true)
            .with_depth_buffer(24)
            .with_vsync(true) // TODO figure out interactions on non-wayland systems
            .with_srgb(true)
            .build_windowed(win_builder, &event_q)?
    };

    let ctx = unsafe {
        ctx.make_current()
            .expect("Error making GL context current")
    };

    gl::load_with(|sym| { ctx.get_proc_address(sym) as *const _ });

    {   let mut major: GLint = 0;
        let mut minor: GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        }
        eprintln!("Using OpenGL {}.{} Core profile", major, minor);
    }

    unsafe {
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DONT_CARE,
            0, ptr::null(),
            gl::TRUE
        );
        gl::DebugMessageCallback(Some(on_gl_debug), ptr::null());
    }

    let mut app = App::new(ctx)?;
    let mut events = Vec::new();
    let mut next_tick = Instant::now() + TICK_INTERVAL;

    event_q.run(move |event, _, flow| {
        let event = match event.to_static() {
            Some(event) => event,
            None        => { return; }
        };

        use event::Event::*;
        match &event {
            NewEvents(_) => {
                events.clear();
            }

            WindowEvent { event: win_event, .. } => {
                use event::WindowEvent::*;
                if *win_event == CloseRequested {
                    *flow = ControlFlow::Exit;
                }
                else {
                    events.push(event);
                }
            }

            DeviceEvent { .. } => {
                events.push(event);
            }

            MainEventsCleared => {
                for event in events.iter() {
                    app.handle_event(event);
                }
                events.clear();

                let now = Instant::now();
                if now >= next_tick {
                    app.tick();
                    next_tick += TICK_INTERVAL;
                    while next_tick <= now {
                        eprintln!("skipping tick");
                        next_tick += TICK_INTERVAL;
                    }
                }

                // TODO use WaitUntil sparingly for better efficiency
                *flow = ControlFlow::Poll;//WaitUntil(next_tick);
            }

            _ => { }
        }
    });
}

