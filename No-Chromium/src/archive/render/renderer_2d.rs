#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub enum Paint2DKind {
    SolidColor,
    Textured,
    Glyph,
}

#[derive(Debug, Clone, Copy)]
pub struct Paint2DCommand {
    pub kind: Paint2DKind,
    pub bounds: [f32; 4],
    pub color: [f32; 4],
}

pub struct Renderer2D {
    commands: Vec<Paint2DCommand>,
}

impl Renderer2D {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: Paint2DCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[Paint2DCommand] {
        &self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
