use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    console, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
};

pub struct Chip8WebGLDisplay {
    gl: WebGl2RenderingContext,
    render_texture: Option<WebGlTexture>,
    // gl_video_buffer: WebGlBuffer,
    pub video_buffer: [u8; 64 * 32],
    program: WebGlProgram,
}

impl Chip8WebGLDisplay {
    pub const CHIP8_DISPLAY_HEIGHT: u8 = 32;
    pub const CHIP8_DISPLAY_WIDTH: u8 = 64;

    // Initialize WebGL environment
    pub fn new(canvas_name: &str) -> Chip8WebGLDisplay {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = match document.get_element_by_id(canvas_name) {
            Some(element) => element,
            None => panic!("No canvas element found with the id provided for the chip8 display"),
        };

        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>();

        let canvas = match canvas {
            Ok(val) => val,
            Err(err) => {
                panic!(
                    "Err: {:?}, could not find a canvas with the corresponding id: {}",
                    &err, canvas_name
                );
            }
        };

        let gl = canvas
            .get_context("webgl2")
            .expect("Failed finding WebGL context")
            .expect("Failed to find a webgl context!")
            .dyn_into::<WebGl2RenderingContext>()
            .expect("Dynamic Cast Fail");

        let vert_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es

            in vec4 position;
    
            void main() {
                gl_Position = position;
            }
            "##,
        )
        .expect("Chip8Display vertex shader compilation error");

        let frag_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es
            precision highp float;
            
            uniform sampler2D uSampler;

            out vec4 frag;
            // in vec4 gl_FragCoord;

            void main() {
                // 800 px (canvas) / 64 px (video buffer) = 12.5 (scaling)
                float x = 1.0 / 64.0 * gl_FragCoord.x / 12.5;
                float y = 1.0 / 32.0 * gl_FragCoord.y / 12.5;
                vec2 some_pos = vec2(x, y);
                float sample_pix = texture(uSampler, some_pos).x;

                frag = vec4(0.0, 0.0, 0.0, sample_pix);
            }
            "##,
        )
        .expect("Chip8Display fragment shader compilation error");

        let program = link_shader_program(&gl, &vert_shader, &frag_shader)
            .expect("Failed linking shaders to WebGL");
        gl.use_program(Some(&program));

        let video_buffer = [0; 64 * 32];

        let mut display = Chip8WebGLDisplay {
            render_texture: None,
            video_buffer: video_buffer,
            gl: gl,
            program: program,
        };

        display
            .init_buffers()
            .expect("Failed initializing WebGL program buffers");

        display
    }

    // fn test(self) {}
    fn init_buffers(&mut self) -> Result<(), JsValue> {
        let gl = &self.gl;
        let program = &self.program;
        // let program =

        let vertices = [
            -1.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0,
            1.0, 0.0,
        ];

        // ATTRIBUTES
        let position_attribute_location = gl.get_attrib_location(&program, "position");
        let u_sampler_attrib = gl.get_uniform_location(&program, "uSampler");

        // gl.uniform1fv(floatUniformLoc, [v]);

        // BUFFERS
        let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vao = gl
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position_attribute_location as u32);

        gl.bind_vertex_array(Some(&vao));

        let vert_count = (vertices.len() / 3) as i32;

        // Some nums
        let colors = [
            1.0, 1.0, 1.0, 1.0, // white
            1.0, 0.0, 0.0, 1.0, // red
            0.0, 1.0, 0.0, 1.0, // green
            0.0, 0.0, 1.0, 1.0, // blue
        ];

        let color_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));

        unsafe {
            let colors = js_sys::Float32Array::view(&colors);

            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &colors,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        // Actually read binary pixel data from chip8 video buffer into a WebGL buffer
        let texture = gl.create_texture();

        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + 0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
        gl.uniform1i(u_sampler_attrib.as_ref(), 0);

        {
            // define size and format of level 0
            let level = 0;
            let internal_format = WebGl2RenderingContext::R8;
            let border = 0;
            let format = WebGl2RenderingContext::RED;
            let gl_type = WebGl2RenderingContext::UNSIGNED_BYTE;
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                level,
                internal_format as i32,
                Chip8WebGLDisplay::CHIP8_DISPLAY_WIDTH as i32,
                Chip8WebGLDisplay::CHIP8_DISPLAY_HEIGHT as i32,
                border,
                format,
                gl_type,
                Some(&self.video_buffer),
            )
            .expect("Failed making texture");

            // set the filtering so we don't need mips
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
        }

        self.render_texture = texture;
        // Probably should not immediatly draw on initialization, but for not it does.
        // draw(&gl, vert_count, &gl_video_buffer, &chip8_video_buffer);
        self.draw();

        Ok(())
    }

    // Clear video buffer
    pub fn clear(&mut self) {
        self.video_buffer = [0; 64 * 32];
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, state: bool) {
        if state {
            self.video_buffer[(y * 64 + x) as usize] = 0xFF;
        } else {
            self.video_buffer[(y * 64 + x) as usize] = 0x00;
        }
    }

    pub fn xor_pixel(&mut self, x: u8, y: u8, pixel_color: u8) -> bool {
        if x > 0
            && x < Chip8WebGLDisplay::CHIP8_DISPLAY_WIDTH
            && y > 0
            && y < Chip8WebGLDisplay::CHIP8_DISPLAY_HEIGHT
        {
            let px_before =
                self.video_buffer[(y * Chip8WebGLDisplay::CHIP8_DISPLAY_WIDTH + x) as usize];
            let px_after = px_before & pixel_color;

            if px_before == 0xFF && px_after == 0x00 {
                true;
            }
        }

        false
    }

    pub fn set_checkerboard(&mut self, state: bool) {
        for y in 0..32 {
            for x in 0..64 {
                if (x + y) % 2 == 0 {
                    if state {
                        self.video_buffer[(y * 64 + x) as usize] = 0xFF;
                    } else {
                        self.video_buffer[(y * 64 + x) as usize] = 0x00;
                    }
                } else {
                    if state {
                        self.video_buffer[(y * 64 + x) as usize] = 0x00;
                    } else {
                        self.video_buffer[(y * 64 + x) as usize] = 0xFF;
                    }
                }
            }
        }
    }

    // Random test function
    pub fn set_some_pixels(&mut self) {
        for x in 20..40 {
            for y in 20..40 {
                self.video_buffer[x * y] = 0xFF;
            }
        }
    }

    // Draw video buffer to webgl buffer to screen
    pub fn draw(&mut self) {
        let gl = &self.gl;
        let vert_count = 6;

        gl.clear_color(0.0, 1.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + 0);
        gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            self.render_texture.as_ref(),
        );
        // gl.uniform1i(u_sampler_attrib.as_ref(), 0);

        let level = 0;
        let internal_format = WebGl2RenderingContext::R8;
        let border = 0;
        let format = WebGl2RenderingContext::RED;
        let gl_type = WebGl2RenderingContext::UNSIGNED_BYTE;

        unsafe {
            console::log_1(&JsValue::from_str("Hello"));
        }

        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,             // target: u32,
            level,                                          // level: i32,
            0,                                              // xoffset: i32,
            0,                                              // yoffset: i32,
            Chip8WebGLDisplay::CHIP8_DISPLAY_WIDTH as i32,  // width: i32,
            Chip8WebGLDisplay::CHIP8_DISPLAY_HEIGHT as i32, // height: i32,
            format,                                         // format: u32,
            gl_type,                                        // type_: u32,
            Some(&self.video_buffer),
        )
        .expect("Failed updating sub texture");

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vert_count);
        // draw(&self.gl, 6);
    }
}

pub fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_shader_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let shader_program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&shader_program, vert_shader);
    gl.attach_shader(&shader_program, frag_shader);
    gl.link_program(&shader_program);

    if gl
        .get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader_program)
    } else {
        Err(gl
            .get_program_info_log(&shader_program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
