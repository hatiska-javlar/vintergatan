use glium::{DrawParameters, Program, Surface, VertexBuffer};
use glium::backend::Facade;
use glium::draw_parameters::Blend;
use glium::index::{NoIndices, PrimitiveType};
use glium::texture::{RawImage2d, SrgbTexture2d};
use image;

#[derive(Copy, Clone)]
struct TextureVertex {
    position: [f32; 2],
    tex_coords: [f32; 2]
}

implement_vertex!(TextureVertex, position, tex_coords);

pub struct GameCursor {
    x: f64,
    y: f64,
    texture: SrgbTexture2d,
    vertex_buffer: VertexBuffer<TextureVertex>,
    index_buffer: NoIndices,
    program: Program,
    draw_params: DrawParameters<'static>
}

impl GameCursor {
    pub fn new(facade: &Facade) -> Self {
        let image = image::load(::std::io::Cursor::new(&include_bytes!("../../assets/cursor.png")[..]), image::PNG).unwrap().to_rgba();
        let image_dimensions = image.dimensions();

        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = SrgbTexture2d::new(facade, image).unwrap();

        let shape = vec![
            TextureVertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] }, // 1
            TextureVertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] }, // 2
            TextureVertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] }, // 3
            TextureVertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] }, // 3
            TextureVertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }, // 4
            TextureVertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] }, // 1
        ];

        GameCursor {
            x: 0.0,
            y: 0.0,
            texture,
            vertex_buffer: VertexBuffer::new(facade, &shape).unwrap(),
            index_buffer: NoIndices(PrimitiveType::TrianglesList),
            program: program!(facade,
                140 => {
                    vertex: r#"
                        #version 140

                        in vec2 position;
                        in vec2 tex_coords;
                        out vec2 v_tex_coords;

                        uniform mat4 matrix;

                        void main() {
                            v_tex_coords = tex_coords;
                            gl_Position = matrix * vec4(position, 0.0, 1.0);
                        }
                    "#,
                    fragment: r#"
                        #version 140

                        in vec2 v_tex_coords;
                        out vec4 color;

                        uniform sampler2D tex;

                        void main() {
                            color = texture(tex, v_tex_coords);
                        }
                    "#
                }
            ).unwrap(),
            draw_params: DrawParameters {
                blend: Blend::alpha_blending(),
                .. Default::default()
            }
        }
    }

    pub fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    pub fn set_position(&mut self, position: (f64, f64)) {
        self.x = position.0;
        self.y = position.1;
    }

    pub fn draw<S>(&self, target: &mut S) where S: Surface {
        let dimensions = target.get_dimensions();

        let width = dimensions.0 as f32;
        let height = dimensions.1 as f32;

        let aspect = height / width;

        let x = (self.x as f32) * 2.0 / width - 1.0;
        let y = -(self.y as f32) * 2.0 / height + 1.0;

        let uniform = uniform! {
            matrix: [
                [    aspect * 0.03,      0.0,  0.0,    0.0],
                [              0.0,     0.03,  0.0,    0.0],
                [              0.0,      0.0, 0.03,    0.0],
                [x + aspect * 0.03, y - 0.03,  0.0, 1.0f32]
            ],
            tex: &self.texture
        };

        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniform, &self.draw_params).unwrap();
    }
}
