extern crate sdl;
extern crate opengles;

use opengles::gl2;
use opengles::gl2::{GLuint, GLint, GLfloat, GLenum, GLsizei, GLushort};
use std::io::{File, BufferedReader};

struct Uniforms {
    fade_factor: GLint,
    textures: [GLint, ..2],
}

struct Attributes {
    position: GLint,
}

struct Resources {
    vertex_buffer: GLuint,
    element_buffer: GLuint,
    textures: [GLuint, ..2],
    program: GLuint,
    uniforms: Uniforms,
    attributes: Attributes,
    fade_factor: GLfloat,
}

fn make_buffer<T>(target: GLenum, data: &[T]) -> GLuint {
    let buffers = gl2::gen_buffers(1);
    let buffer = match buffers.len() {
        0 => fail!("couldn't create buffer"),
        _ => buffers[0],
    };
    gl2::bind_buffer(target, buffer);
    gl2::buffer_data(target, data, gl2::STATIC_DRAW);
    buffer
}

static RGB_PIXFMT: sdl::video::PixelFormat = sdl::video::PixelFormat {
    palette: None,
    bpp: 24,
    r_loss: 0,
    g_loss: 0,
    b_loss: 0,
    a_loss: 0,
    r_shift: 16,
    g_shift: 8,
    b_shift: 0,
    a_shift: 0,
    r_mask: 0xff,
    g_mask: 0xff00,
    b_mask: 0xff0000,
    a_mask: 0,
    color_key: 0,
    alpha: 0,
};

fn make_texture(filename: &str) -> GLuint {
    let path = Path::new(filename);
    let bmp = match sdl::video::Surface::from_bmp(&path) {
        Err(err) => fail!(format!("couldn't load {:s}: {:s}", filename, err)),
        Ok(s) => s,
    };
    let rgb = match bmp.convert(&RGB_PIXFMT, [sdl::video::SWSurface]) {
        Err(err) => fail!(format!("couldn't convert {:s} to RGB: {:s}", filename, err)),
        Ok(s) => s,
    };

    let textures = gl2::gen_textures(1);
    let texture = match textures.len() {
        0 => fail!("couldn't create texture"),
        _ => textures[0],
    };
    gl2::bind_texture(gl2::TEXTURE_2D, texture);
    gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MIN_FILTER, gl2::LINEAR as GLint);
    gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MAG_FILTER, gl2::LINEAR as GLint);
    gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_S,     gl2::CLAMP_TO_EDGE as GLint);
    gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_T,     gl2::CLAMP_TO_EDGE as GLint);

    rgb.with_lock(|pixels| {
        gl2::tex_image_2d(
            gl2::TEXTURE_2D, 0,
            gl2::RGB as GLint,
            rgb.get_width() as GLsizei, rgb.get_height() as GLsizei, 0,
            gl2::RGB, gl2::UNSIGNED_BYTE,
            Some(unsafe { std::mem::transmute(pixels) }) // XXX there's got to be a better way
        );
    });

    texture
}

fn make_shader(ty: GLenum, filename: &str) -> GLuint {
    let path = Path::new(filename);
    let mut r = BufferedReader::new(File::open(&path));
    let source = match r.read_to_end() {
        Err(err) => fail!(format!("couldn't load shader {:s}: {:s}", filename, err.desc)),
        Ok(s) => s,
    };

    let shader = gl2::create_shader(ty);
    gl2::shader_source(shader, &[source.as_slice()]);
    gl2::compile_shader(shader);

    match gl2::get_shader_iv(shader, gl2::COMPILE_STATUS) {
        0 => fail!(format!("failed to compile {:s}: {:s}", filename, gl2::get_shader_info_log(shader))),
        _ => {}
    };

    shader
}

fn make_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = gl2::create_program();
    gl2::attach_shader(program, vertex_shader);
    gl2::attach_shader(program, fragment_shader);
    gl2::link_program(program);

    match gl2::get_program_iv(program, gl2::LINK_STATUS) {
        0 => fail!(format!("failed to link shader program: {:s}", gl2::get_program_info_log(program))),
        _ => {}
    };

    program
}

static VERTEX_BUFFER_DATA: [GLfloat, ..8] = [
    -1.0, -1.0,
     1.0, -1.0,
    -1.0,  1.0,
     1.0,  1.0
];

static ELEMENT_BUFFER_DATA: [GLushort, ..4] = [ 0, 1, 2, 3 ];

fn make_resources() -> Option<Resources> {
    let program = make_program(
        make_shader(gl2::VERTEX_SHADER, "hello-gl.v.glsl"),
        make_shader(gl2::FRAGMENT_SHADER, "hello-gl.f.glsl")
    );

    Some(Resources {
        vertex_buffer: make_buffer(gl2::ARRAY_BUFFER, VERTEX_BUFFER_DATA),
        element_buffer: make_buffer(gl2::ELEMENT_ARRAY_BUFFER, ELEMENT_BUFFER_DATA),
        textures: [
            make_texture("hello1.bmp"),
            make_texture("hello2.bmp"),
        ],
        program: program,
        uniforms: Uniforms {
            fade_factor: gl2::get_uniform_location(program, "fade_factor"),
            textures: [
                gl2::get_uniform_location(program, "textures[0]"),
                gl2::get_uniform_location(program, "textures[1]"),
            ],
        },
        attributes: Attributes {
            position: gl2::get_attrib_location(program, "position"),
        },
        fade_factor: 0.0,
    })
}

fn update_fade_factor(rsrc: &mut Resources) {
    let ms = sdl::get_ticks() as f32;
    rsrc.fade_factor = ((ms * 0.001).sin() * 0.5 + 0.5) as GLfloat;
}

fn render(rsrc: &Resources) {
    gl2::use_program(rsrc.program);

    gl2::uniform_1f(rsrc.uniforms.fade_factor, rsrc.fade_factor);

    gl2::active_texture(gl2::TEXTURE0);
    gl2::bind_texture(gl2::TEXTURE_2D, rsrc.textures[0]);
    gl2::uniform_1i(rsrc.uniforms.textures[0], 0);

    gl2::active_texture(gl2::TEXTURE1);
    gl2::bind_texture(gl2::TEXTURE_2D, rsrc.textures[1]);
    gl2::uniform_1i(rsrc.uniforms.textures[1], 1);

    gl2::bind_buffer(gl2::ARRAY_BUFFER, rsrc.vertex_buffer);
    gl2::vertex_attrib_pointer_f32(
        rsrc.attributes.position as GLuint,
        2,
        false,
        8, // XXX sizeof(GLfloat)*2
        0);
    gl2::enable_vertex_attrib_array(rsrc.attributes.position as GLuint);

    gl2::bind_buffer(gl2::ELEMENT_ARRAY_BUFFER, rsrc.element_buffer);
    gl2::draw_elements(gl2::TRIANGLE_STRIP, 4, gl2::UNSIGNED_SHORT, None);

    gl2::disable_vertex_attrib_array(rsrc.attributes.position as GLuint);

    sdl::gl::swap_buffers();
}

fn main() {
    #![main]

    sdl::init([sdl::InitVideo]);

    let info = sdl::video::get_video_info();
    let (rs, gs, bs) = match info.format.bpp {
        16      => (5, 6, 5),
        24 | 32 => (8, 8, 8),
        _       => fail!(format!("invalid pixel depth: {:d} bpp", info.format.bpp as int))
    };

    sdl::gl::set_attribute(sdl::gl::RedSize, rs);
    sdl::gl::set_attribute(sdl::gl::GreenSize, gs);
    sdl::gl::set_attribute(sdl::gl::BlueSize, bs);
    sdl::gl::set_attribute(sdl::gl::DepthSize, 24);
    sdl::gl::set_attribute(sdl::gl::DoubleBuffer, 1);
    sdl::gl::set_attribute(sdl::gl::SwapControl, 1);

    match sdl::video::set_video_mode(400, 300, info.format.bpp as int, [], [sdl::video::OpenGL]) {
        Ok(_)    => {},
        Err(err) => fail!(format!("failed to set video mode: {:s}", err))
    };

    sdl::wm::set_caption("Hello World", "Hello World");

    let mut rsrc = match make_resources() {
        Some(r) => r,
        None    => fail!("failed to load resources")
    };

    'main: loop {
        'event: loop {
            match sdl::event::poll_event() {
                sdl::event::QuitEvent => break 'main,
                sdl::event::NoEvent   => break 'event,
                sdl::event::KeyEvent(sdl::event::EscapeKey, true, _, _) => break 'main,
                _                     => {}
            }
        }

        update_fade_factor(&mut rsrc);
        render(&rsrc);
    }

    sdl::quit();
}
