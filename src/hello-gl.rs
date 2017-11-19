extern crate sdl2;
extern crate gleam;

use sdl2::video::{GLProfile};
use sdl2::surface::Surface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::keyboard::Keycode;
use gleam::gl;
use gleam::gl::types::{GLuint, GLint, GLfloat, GLenum, GLsizei, GLushort};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::mem;

struct Uniforms {
    fade_factor: GLint,
    textures: [GLint; 2],
}

struct Attributes {
    position: GLint,
}

struct Resources {
    vertex_buffer: GLuint,
    element_buffer: GLuint,
    textures: [GLuint; 2],
    program: GLuint,
    uniforms: Uniforms,
    attributes: Attributes,
    fade_factor: GLfloat,
}

type GlPtr = std::rc::Rc<gl::Gl>;

fn make_buffer<T>(gl: &GlPtr, target: GLenum, data: &[T]) -> GLuint {
    let buffers = gl.gen_buffers(1);
    let buffer = match buffers.len() {
        0 => panic!("couldn't create buffer"),
        _ => buffers[0],
    };
    gl.bind_buffer(target, buffer);
    gl.buffer_data_untyped(target, mem::size_of_val(data) as isize, data.as_ptr() as *const _, gl::STATIC_DRAW);
    buffer
}

fn make_texture(gl: &GlPtr, filename: &str) -> GLuint {
    let path = Path::new(filename);
    let bmp = match Surface::load_bmp(&path) {
        Ok(s) => s,
        Err(err) => panic!("couldn't load {}: {}", filename, err),
    };
    let fmt = Surface::new(1,1,PixelFormatEnum::RGB24).unwrap().pixel_format();
    let rgb = match bmp.convert(&fmt) {
        Ok(s) => s,
        Err(err) => panic!("couldn't convert {} to RGB: {}", filename, err),
    };

    let textures = gl.gen_textures(1);
    let texture = match textures.len() {
        0 => panic!("couldn't create texture"),
        _ => textures[0],
    };
    gl.bind_texture(gl::TEXTURE_2D, texture);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S,     gl::CLAMP_TO_EDGE as GLint);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T,     gl::CLAMP_TO_EDGE as GLint);

    let width = rgb.width();
    let height = rgb.height();
    rgb.with_lock(|pixels| {
        gl.tex_image_2d(
            gl::TEXTURE_2D, 0,
            gl::RGB as GLint,
            width as GLsizei, height as GLsizei, 0,
            gl::RGB, gl::UNSIGNED_BYTE,
            Some(pixels.as_ref())
        );
    });

    texture
}

fn make_shader(gl: &GlPtr, typ: GLenum, filename: &str) -> GLuint {
    let path = Path::new(filename);
    let file = match File::open(path) {
        Ok(f)    => f,
        Err(err) => panic!("couldn't open shader {}: {}", filename, err),
    };
    let mut r = BufReader::new(file);
    let mut source: Vec<u8> = Vec::new();
    match r.read_to_end(&mut source) {
        Ok(_)    => (),
        Err(err) => panic!("couldn't read shader {}: {}", filename, err),
    };

    let shader = match gl.create_shader(typ) {
        0 => panic!("couldn't create shader object: {}", gl.get_error()),
        s => s,
    };
    gl.shader_source(shader, &[source.as_slice()]);
    gl.compile_shader(shader);

    match gl.get_shader_iv(shader, gl::COMPILE_STATUS) {
        0 => panic!("failed to compile {}: {}", filename, gl.get_shader_info_log(shader)),
        _ => ()
    };

    shader
}

fn make_program(gl: &GlPtr, vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = gl.create_program();
    gl.attach_shader(program, vertex_shader);
    gl.attach_shader(program, fragment_shader);
    gl.link_program(program);

    match gl.get_program_iv(program, gl::LINK_STATUS) {
        0 => panic!("failed to link shader program: {}", gl.get_program_info_log(program)),
        _ => ()
    };

    program
}

static VERTEX_BUFFER_DATA: [GLfloat; 8] = [
    -1.0, -1.0,
     1.0, -1.0,
    -1.0,  1.0,
     1.0,  1.0
];

static ELEMENT_BUFFER_DATA: [GLushort; 4] = [ 0, 1, 2, 3 ];

fn make_resources(gl: &GlPtr) -> Option<Resources> {
    let program = make_program(
        gl,
        make_shader(gl, gl::VERTEX_SHADER, "hello-gl.v.glsl"),
        make_shader(gl, gl::FRAGMENT_SHADER, "hello-gl.f.glsl")
    );

    Some(Resources {
        vertex_buffer: make_buffer(gl, gl::ARRAY_BUFFER, &VERTEX_BUFFER_DATA),
        element_buffer: make_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &ELEMENT_BUFFER_DATA),
        textures: [
            make_texture(gl, "hello1.bmp"),
            make_texture(gl, "hello2.bmp"),
        ],
        program: program,
        uniforms: Uniforms {
            fade_factor: gl.get_uniform_location(program, "fade_factor"),
            textures: [
                gl.get_uniform_location(program, "textures[0]"),
                gl.get_uniform_location(program, "textures[1]"),
            ],
        },
        attributes: Attributes {
            position: gl.get_attrib_location(program, "position"),
        },
        fade_factor: 0.0,
    })
}

fn update_fade_factor(sdl_ctx: &sdl2::Sdl, rsrc: &mut Resources) {
    let ms = sdl_ctx.timer().unwrap().ticks() as f32;
    rsrc.fade_factor = ((ms * 0.001).sin() * 0.5 + 0.5) as GLfloat;
}

fn render(gl: &GlPtr, rsrc: &Resources) {
    gl.use_program(rsrc.program);

    gl.uniform_1f(rsrc.uniforms.fade_factor, rsrc.fade_factor);

    gl.active_texture(gl::TEXTURE0);
    gl.bind_texture(gl::TEXTURE_2D, rsrc.textures[0]);
    gl.uniform_1i(rsrc.uniforms.textures[0], 0);

    gl.active_texture(gl::TEXTURE1);
    gl.bind_texture(gl::TEXTURE_2D, rsrc.textures[1]);
    gl.uniform_1i(rsrc.uniforms.textures[1], 1);

    gl.bind_buffer(gl::ARRAY_BUFFER, rsrc.vertex_buffer);
    gl.vertex_attrib_pointer_f32(
        rsrc.attributes.position as GLuint,
        2,
        false,
        (mem::size_of::<GLuint>()*2) as GLsizei,
        0);
    gl.enable_vertex_attrib_array(rsrc.attributes.position as GLuint);

    gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, rsrc.element_buffer);
    gl.draw_elements(gl::TRIANGLE_STRIP, 4, gl::UNSIGNED_SHORT, 0);

    gl.disable_vertex_attrib_array(rsrc.attributes.position as GLuint);
}

#[allow(unused_variables)]
fn main() {
    let sdl_ctx = match sdl2::init() {
        Ok(ctx)  => ctx,
        Err(err) => panic!("failed to create SDL context: {}", err),
    };
    let video_subsystem = sdl_ctx.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(GLProfile::GLES);
    gl_attr.set_context_version(2, 0);
    gl_attr.set_red_size(8);
    gl_attr.set_green_size(8);
    gl_attr.set_blue_size(8);
    gl_attr.set_depth_size(24);
    gl_attr.set_double_buffer(true);

    let window = match video_subsystem.window("Hello GL!", 400, 300)
        .position_centered().opengl().build() {
        Ok(window) => window,
        Err(err)   => panic!("failed to create window: {}", err),
    };

    let gl_context = match window.gl_create_context() {
        Ok(ctx)  => ctx,
        Err(err) => panic!("failed to create GL context: {}", err),
    };

    let gl = unsafe { gl::GlesFns::load_with(|s| {
        mem::transmute(video_subsystem.gl_get_proc_address(s))
    })};

    let mut rsrc = match make_resources(&gl) {
        Some(r) => r,
        None    => panic!("failed to load resources"),
    };

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    'main: loop {
        'event: for event in event_pump.poll_iter() {
            use sdl2::event::Event;

            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
                _ => (),
            };
        }

        update_fade_factor(&sdl_ctx, &mut rsrc);
        render(&gl, &rsrc);

        window.gl_swap_window();
    }
}
