use crate::{chip8::Chip8, SCALE};
use tetra::{
    graphics::{
        self,
        mesh::{GeometryBuilder, ShapeStyle},
        Color, DrawParams, Rectangle,
    },
    input::{self, Key},
    math::Vec2,
    Context, State,
};

pub struct System {
    pub chip8: Chip8,
    needs_draw: bool,
}

impl System {
    pub fn new(chip8: Chip8) -> System {
        System {
            chip8,
            needs_draw: false,
        }
    }
}

impl State for System {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let is_pressed = |key: Key| input::is_key_down(ctx, key);
        if !self.chip8.cycle(is_pressed) {
            // return Ok(());
        }

        graphics::clear(ctx, Color::BLACK);

        let mut builder = GeometryBuilder::new();
        let style = ShapeStyle::Fill;
        let color = Color::WHITE;
        let pixels = self.chip8.pixels();
        for x in 0..64 {
            for y in 0..32 {
                if pixels[x + (y * 64)] {
                    let bounds = Rectangle::new(x as f32 * SCALE, y as f32 * SCALE, SCALE, SCALE);
                    builder.rectangle(style, bounds)?;
                }
            }
        }
        let mesh = builder.build_mesh(ctx)?;
        mesh.draw(
            ctx,
            DrawParams::new().position(Vec2::new(0.0, 0.0)).color(color),
        );

        self.needs_draw = false;
        Ok(())
    }
}
