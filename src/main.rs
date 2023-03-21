// Prevent console spawn on start on Windwos
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod obj;

use obj::*;

use std::{
    io::Write,
    mem::{size_of, size_of_val},
    ops::ControlFlow,
};

use {
    gl::load_with,
    glfw::{Action, Context, Key, Modifiers, SwapInterval},
};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(300, 300, "[debug] learn-opengl", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);

    // Load every OpenGL function
    load_with(|f_name| window.get_proc_address(f_name));

    let object = obj::Object::load("objects/sphere.obj").unwrap();
    let (vertices, normals) = (object.vertices(), object.normals());

    // Set clear (background) color
    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
    }

    // Generate a Vertex Array Object
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }
    assert_ne!(vao, 0);

    // Generate a Vertex Buffer Object
    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);

        // Bind it
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        // Pass data to it
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * size_of_val(&vertices[0])) as isize,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
    }

    // Enable vertex attribute: positions
    unsafe {
        gl::VertexAttribPointer(
            // Index
            0,
            // Component count
            vertices[0].len() as i32,
            // Component type
            gl::FLOAT,
            // Normalized?
            gl::FALSE,
            // Stride (could also be 0 here)
            size_of::<Coords>().try_into().unwrap(),
            // Pointer in VBO
            0 as *const _,
        );

        gl::EnableVertexAttribArray(0);
    }

    // Enable vertex attribute: normals
    unsafe {
        gl::VertexAttribPointer(
            // Index
            1,
            // Component count
            normals[0].len() as i32,
            // Component type
            gl::FLOAT,
            // Normalized?
            gl::FALSE,
            // Stride (could also be 0 here)
            size_of::<Coords>().try_into().unwrap(),
            // Pointer in VBO
            0 as *const _,
        );

        gl::EnableVertexAttribArray(1);
    }

    let vertex = setup_vertex_shader();
    let fragment = setup_fragment_shader();

    unsafe {
        let program = gl::CreateProgram();

        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);

        check_program_error(program);

        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);

        gl::UseProgram(program);
    }

    window.glfw.set_swap_interval(SwapInterval::Sync(1));

    // Loop until the user closes the window
    while !window.should_close() {
        // Swap front and back buffers
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, (vertices.len() * 3) as i32);
            window.swap_buffers();
        }

        // Poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            if let ControlFlow::Break(_) = echo_events(event, &mut window) {
                continue;
            }

            std::io::stdout().flush().unwrap();
        }
    }

    println!("");
}

fn setup_vertex_shader() -> u32 {
    unsafe {
        let shader = gl::CreateShader(gl::VERTEX_SHADER);
        assert_ne!(shader, 0);

        let shader_code = include_str!("shaders/vertex.glsl");

        compile_shader(shader, shader_code);

        check_shader_error(shader, "Vertex");

        shader
    }
}

fn setup_fragment_shader() -> u32 {
    unsafe {
        let shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        assert_ne!(shader, 0);

        let shader_code = include_str!("shaders/fragment.glsl");

        compile_shader(shader, shader_code);

        check_shader_error(shader, "Fragment");

        shader
    }
}

fn compile_shader(shader: u32, shader_code: &str) {
    unsafe {
        gl::ShaderSource(
            shader,
            1,
            &(shader_code.as_bytes().as_ptr().cast()),
            &(shader_code.len().try_into().unwrap()),
        );

        gl::CompileShader(shader);
    }
}

fn check_shader_error(shader: u32, shader_type: &str) {
    unsafe {
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut log_length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut vec = Vec::<u8>::with_capacity(log_length as usize);
            let mut returned_log_length = 0;
            gl::GetShaderInfoLog(
                shader,
                log_length,
                &mut returned_log_length,
                vec.as_mut_ptr().cast(),
            );

            vec.set_len(returned_log_length.try_into().unwrap());

            panic!(
                "{shader_type} compile error: {}",
                String::from_utf8_lossy(&vec)
            )
        }
    }
}

fn check_program_error(program: u32) {
    unsafe {
        let mut success = 0;
        gl::GetShaderiv(program, gl::LINK_STATUS, &mut success);

        if success == 0 {
            let mut log_length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut vec = Vec::<u8>::with_capacity(log_length as usize);
            let mut returned_log_length = 0;
            gl::GetProgramInfoLog(
                program,
                log_length,
                &mut returned_log_length,
                vec.as_mut_ptr().cast(),
            );

            vec.set_len(returned_log_length.try_into().unwrap());

            // panic!(
            //     "Program link error: {} ({vec:?})",
            //     String::from_utf8_lossy(&vec)
            // )
        }
    }
}

fn echo_events(event: glfw::WindowEvent, window: &mut glfw::Window) -> ControlFlow<()> {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::Key(key, _, Action::Press, modifiers)
            if (Key::A..Key::Z).contains(&key)
                && (modifiers.is_empty()
                    || (modifiers & (Modifiers::Shift | Modifiers::CapsLock)).bits() != 0) =>
        {
            if modifiers.is_empty() {
                print!("{}", format!("{key:?}").to_lowercase());
            } else if !(modifiers & (Modifiers::CapsLock | Modifiers::Shift)).is_empty() {
                print!("{key:?}")
            }
        }
        glfw::WindowEvent::Key(key, _, Action::Press, modifiers) => {
            let key = match key {
                Key::Space => " ",
                Key::Enter => "\n",
                Key::Comma => ",",
                Key::Period => ".",
                Key::Apostrophe => "'",
                Key::Backspace => "\x1b[1D \x1b[1D",
                _ => return ControlFlow::Break(()),
            };
            print!(
                "{}",
                if (modifiers & (Modifiers::Shift | Modifiers::CapsLock)).bits() != 0 {
                    key.to_uppercase()
                } else {
                    key.to_owned()
                }
            );
        }
        _ => {}
    }

    ControlFlow::Continue(())
}
