extern crate sdl2;
extern crate gleam;
extern crate cgmath;

use sdl2::video::{GLProfile};
use sdl2::keyboard::Keycode;
use gleam::gl;
use gleam::gl::types::{GLuint, GLint, GLfloat, GLenum, GLsizei, GLushort};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::mem;
use cgmath::{Matrix3,Matrix4,frustum,vec3,Deg};

struct Uniforms {
    modelview_matrix: GLint,
    modelviewprojection_matrix: GLint,
    normal_matrix: GLint,
}

struct Attributes {
    position: GLint,
    color: GLint,
    normal: GLint,
}

struct Resources {
    vertex_buffer: GLuint,
    element_buffer: GLuint,
    program: GLuint,
    uniforms: Uniforms,
    attributes: Attributes,
    i: GLint,
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

/* Cube vertex data */
static VERTEX_BUFFER_DATA: [GLfloat; 3*3*4*6] = [
            // front
            -1.0, -1.0, 1.0, // point blue
            0.0,  0.0,  1.0, // blue
            0.0, 0.0, 1.0, // forward

            1.0, -1.0, 1.0, // point magenta
            1.0,  0.0,  1.0, // magenta
            0.0, 0.0, 1.0, // forward

            -1.0, 1.0, 1.0, // point cyan
            0.0,  1.0,  1.0, // cyan
            0.0, 0.0, 1.0, // forward

            1.0, 1.0, 1.0, // point white
            1.0,  1.0,  1.0, // white
            0.0, 0.0, 1.0, // forward

            // back
            1.0, -1.0, -1.0, // point red
            1.0,  0.0,  0.0, // red
            0.0, 0.0, -1.0, // backbard

            -1.0, -1.0, -1.0, // point black
            0.0,  0.0,  0.0, // black
            0.0, 0.0, -1.0, // backbard

            1.0, 1.0, -1.0, // point yellow
            1.0,  1.0,  0.0, // yellow
            0.0, 0.0, -1.0, // backbard

            -1.0, 1.0, -1.0, // point green
            0.0,  1.0,  0.0, // green
            0.0, 0.0, -1.0, // backbard

            // right
            1.0, -1.0, 1.0, // point magenta
            1.0,  0.0,  1.0, // magenta
            1.0, 0.0, 0.0, // right

            1.0, -1.0, -1.0, // point red
            1.0,  0.0,  0.0, // red
            1.0, 0.0, 0.0, // right

            1.0, 1.0, 1.0, // point white
            1.0,  1.0,  1.0, // white
            1.0, 0.0, 0.0, // right

            1.0, 1.0, -1.0, // point yellow
            1.0,  1.0,  0.0, // yellow
            1.0, 0.0, 0.0, // right

            // left
            -1.0, -1.0, -1.0, // point black
            0.0,  0.0,  0.0, // black
            -1.0, 0.0, 0.0, // left

            -1.0, -1.0, 1.0, // point blue
            0.0,  0.0,  1.0, // blue
            -1.0, 0.0, 0.0, // left

            -1.0, 1.0, -1.0, // point green
            0.0,  1.0,  0.0, // green
            -1.0, 0.0, 0.0, // left

            -1.0, 1.0, 1.0, // point cyan
            0.0,  1.0,  1.0, // cyan
            -1.0, 0.0, 0.0, // left

            // top
            -1.0, 1.0, 1.0, // point cyan
            0.0,  1.0,  1.0, // cyan
            0.0, 1.0, 0.0, // up

            1.0, 1.0, 1.0, // point white
            1.0,  1.0,  1.0, // white
            0.0, 1.0, 0.0, // up

            -1.0, 1.0, -1.0, // point green
            0.0,  1.0,  0.0, // green
            0.0, 1.0, 0.0, // up

            1.0, 1.0, -1.0, // point yellow
            1.0,  1.0,  0.0, // yellow
            0.0, 1.0, 0.0, // up

            // bottom
            -1.0, -1.0, -1.0, // point black
            0.0,  0.0,  0.0, // black
            0.0, -1.0, 0.0, // down

            1.0, -1.0, -1.0, // point red
            1.0,  0.0,  0.0, // red
            0.0, -1.0, 0.0, // down

            -1.0, -1.0, 1.0, // point blue
            0.0,  0.0,  1.0, // blue
            0.0, -1.0, 0.0, // down

            1.0, -1.0, 1.0,  // point magenta
            1.0,  0.0,  1.0,  // magenta
            0.0, -1.0, 0.0,  // down
];

/* Index buffer with degenerate triangles */
static ELEMENT_BUFFER_DATA: [GLushort; 4*6 + 2*5] = [
    0, 1, 2, 3, 3, 4,
    4, 5, 6, 7, 7, 8,
    8, 9, 10, 11, 11, 12,
    12, 13, 14, 15, 15, 16,
    16, 17, 18, 19, 19, 20,
    20, 21, 22, 23,
];

fn make_resources(gl: &GlPtr) -> Option<Resources> {
    let program = make_program(
        gl,
        make_shader(gl, gl::VERTEX_SHADER, "cube.v.glsl"),
        make_shader(gl, gl::FRAGMENT_SHADER, "cube.f.glsl")
    );

    let rsrc = Resources {
        vertex_buffer: make_buffer(gl, gl::ARRAY_BUFFER, &VERTEX_BUFFER_DATA),
        element_buffer: make_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &ELEMENT_BUFFER_DATA),
        program: program,
        uniforms: Uniforms {
            modelview_matrix: gl.get_uniform_location(program, "modelviewMatrix"),
            modelviewprojection_matrix: gl.get_uniform_location(program, "modelviewprojectionMatrix"),
            normal_matrix: gl.get_uniform_location(program, "normalMatrix"),
        },
        attributes: Attributes {
            position: gl.get_attrib_location(program, "in_position"),
            color: gl.get_attrib_location(program, "in_color"),
            normal: gl.get_attrib_location(program, "in_normal"),
        },
        i:0,
    };

    // Set up buffers
    gl.bind_buffer(gl::ARRAY_BUFFER, rsrc.vertex_buffer);
    gl.vertex_attrib_pointer_f32(
        rsrc.attributes.position as GLuint,
        3,
        false,
        (mem::size_of::<GLfloat>()*9) as GLsizei,
        (mem::size_of::<GLfloat>()*0) as u32);
    gl.vertex_attrib_pointer_f32(
        rsrc.attributes.color as GLuint,
        3,
        false,
        (mem::size_of::<GLfloat>()*9) as GLsizei,
        (mem::size_of::<GLfloat>()*3) as u32);
    gl.vertex_attrib_pointer_f32(
        rsrc.attributes.normal as GLuint,
        3,
        false,
        (mem::size_of::<GLfloat>()*9) as GLsizei,
        (mem::size_of::<GLfloat>()*6) as u32);

    Some(rsrc)
}

fn update(_sdl_ctx: &sdl2::Sdl, rsrc: &mut Resources) {
    // let ms = sdl_ctx.timer().unwrap().ticks() as f32;
    // rsrc.fade_factor = ((ms * 0.001).sin() * 0.5 + 0.5) as GLfloat;
    rsrc.i = rsrc.i + 1;
}

fn render(gl: &GlPtr, rsrc: &Resources, width: GLint, height: GLint) {
    gl.enable(gl::CULL_FACE);
    gl.viewport(0, 0, width, height);

	gl.clear_color(0.2, 0.2, 0.2, 1.0);
	gl.clear(gl::COLOR_BUFFER_BIT);

    gl.use_program(rsrc.program);

    let aspect = (height as GLfloat) / (width as GLfloat);
    let i = rsrc.i as GLfloat;

    let modelview: Matrix4<GLfloat> =
        Matrix4::from_translation(vec3(0.0, 0.0, -8.0)) *
        Matrix4::from_axis_angle(vec3(1.0, 0.0, 0.0), Deg(-(45.0 + (0.25 * i)))) *
        Matrix4::from_axis_angle(vec3(0.0, 1.0, 0.0), Deg(-(45.0 - (0.5 * i)))) *
        Matrix4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(-(10.0 + (0.15 * i))));
    let projection: Matrix4<GLfloat> = frustum(-2.8, 2.8, -2.8 * aspect, 2.8 * aspect, 6.0, 10.0);
    let modelviewprojection: Matrix4<GLfloat> = projection * modelview;
    let normal: Matrix3<GLfloat> = Matrix3::new(
                        modelview[0][0], modelview[0][1], modelview[0][2],
                        modelview[1][0], modelview[1][1], modelview[1][2],
                        modelview[2][0], modelview[2][1], modelview[2][2]);

    gl.uniform_matrix_4fv(rsrc.uniforms.modelview_matrix, false,
                          modelview.as_ref() as &[GLfloat; 16]);
    gl.uniform_matrix_4fv(rsrc.uniforms.modelviewprojection_matrix, false,
                          modelviewprojection.as_ref() as &[GLfloat; 16]);
    gl.uniform_matrix_3fv(rsrc.uniforms.normal_matrix, false,
                          normal.as_ref() as &[GLfloat; 9]);

    gl.enable_vertex_attrib_array(rsrc.attributes.position as GLuint);
    gl.enable_vertex_attrib_array(rsrc.attributes.color as GLuint);
    gl.enable_vertex_attrib_array(rsrc.attributes.normal as GLuint);

    gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, rsrc.element_buffer);
    gl.draw_elements(gl::TRIANGLE_STRIP, ELEMENT_BUFFER_DATA.len() as i32, gl::UNSIGNED_SHORT, 0);

    gl.disable_vertex_attrib_array(rsrc.attributes.position as GLuint);
    gl.disable_vertex_attrib_array(rsrc.attributes.color as GLuint);
    gl.disable_vertex_attrib_array(rsrc.attributes.normal as GLuint);
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
    gl_attr.set_depth_size(0);
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

        update(&sdl_ctx, &mut rsrc);
        let size = window.size();
        render(&gl, &rsrc, size.0 as i32, size.1 as i32);

        window.gl_swap_window();
    }
}
