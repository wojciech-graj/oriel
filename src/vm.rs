// Copyright (C) 2023  Wojciech Graj
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

use std::collections::HashMap;

use thiserror::Error;

use crate::{cfg, ir};

impl ir::LogicalOperator {
    fn cmp(&self, i1: u16, i2: u16) -> bool {
        match self {
            ir::LogicalOperator::Equal => i1 == i2,
            ir::LogicalOperator::Less => i1 < i2,
            ir::LogicalOperator::Greater => i1 > i2,
            ir::LogicalOperator::LEqual => i1 <= i2,
            ir::LogicalOperator::GEqual => i1 >= i2,
            ir::LogicalOperator::NEqual => i1 != i2,
        }
    }
}

impl ir::MathOperator {
    fn eval(&self, i1: u16, i2: u16) -> Option<u16> {
        (match self {
            ir::MathOperator::Add => u16::checked_add,
            ir::MathOperator::Subtract => u16::checked_sub,
            ir::MathOperator::Multiply => u16::checked_mul,
            ir::MathOperator::Divide => u16::checked_div,
        })(i1, i2)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Key {
    Virtual(ir::VirtualKey),
    Physical(ir::PhysicalKey),
}

#[derive(Debug, Clone, Copy)]
pub struct MouseRegion<'a> {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
    pub callbacks: &'a ir::MouseCallbacks<'a>,
}

#[derive(Debug, Clone)]
pub struct MenuCategory<'a> {
    pub item: MenuItem<'a>,
    pub members: Vec<MenuMember<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct MenuItem<'a> {
    pub name: &'a str,
    pub label: Option<ir::Identifier<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuMember<'a> {
    Item(MenuItem<'a>),
    Separator,
}

pub enum Input<'a> {
    End,
    Goto(ir::Identifier<'a>),
    Mouse {
        callbacks: &'a ir::MouseCallbacks<'a>,
        x: u16,
        y: u16,
    },
}

pub trait VMSys<'a> {
    fn beep(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_arc(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        x3: u16,
        y3: u16,
        x4: u16,
        y4: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_background(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_bitmap(
        &mut self,
        x: u16,
        y: u16,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_chord(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        x3: u16,
        y3: u16,
        x4: u16,
        y4: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_ellipse(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_flood(
        &mut self,
        x: u16,
        y: u16,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_line(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_number(&mut self, x: u16, y: u16, n: u16) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_pie(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        x3: u16,
        y3: u16,
        x4: u16,
        y4: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_rectangle(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_round_rectangle(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        x3: u16,
        y3: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_sized_bitmap(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw_text(&mut self, x: u16, y: u16, text: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn message_box(
        &mut self,
        typ: ir::MessageBoxType,
        default_button: u16,
        icon: ir::MessageBoxIcon,
        text: &str,
        caption: &str,
    ) -> Result<u16, Box<dyn std::error::Error>>;
    fn run(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn set_keyboard(
        &mut self,
        params: HashMap<Key, ir::Identifier<'a>>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn set_menu(&mut self, menu: &[MenuCategory<'a>]) -> Result<(), Box<dyn std::error::Error>>;
    fn set_mouse(&mut self, regions: &[MouseRegion<'a>]) -> Result<(), Box<dyn std::error::Error>>;
    fn set_wait_mode(&mut self, mode: ir::WaitMode) -> Result<(), Box<dyn std::error::Error>>;
    fn set_window(&mut self, option: ir::SetWindowOption)
        -> Result<(), Box<dyn std::error::Error>>;
    fn use_background(
        &mut self,
        option: ir::BackgroundTransparency,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn use_brush(
        &mut self,
        option: ir::BrushType,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn use_caption(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn use_coordinates(
        &mut self,
        option: ir::Coordinates,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn use_font(
        &mut self,
        name: &str,
        width: u16,
        height: u16,
        bold: ir::FontWeight,
        italic: ir::FontSlant,
        underline: ir::FontUnderline,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn use_pen(
        &mut self,
        option: ir::PenType,
        width: u16,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn wait_input(
        &mut self,
        milliseconds: Option<u16>,
    ) -> Result<Option<Input<'a>>, Box<dyn std::error::Error>>;
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Call stack exhausted")]
    CallStackExhaustedError,
    #[error("Integer Under/Over-flow")]
    MathOperationError,
    #[error("Invalid Virtual Key")]
    InvalidVirtualKeyError,
    #[error("Invalid Physical Key")]
    InvalidPhysicalKeyError,
    #[error("Nonexistent Label")]
    NonexistentLabelError,
    #[error("Number of integer variables exceeds 500")]
    ExcessVariablesError,
    #[error("Number of string variables exceeds 200")]
    ExcessVariablesStrError,
    #[error("System Error: {}", .0)]
    SystemError(#[from] Box<dyn std::error::Error>),
}

macro_rules! incr_ip {
    ($self:ident, $e:expr) => {{
        $e;
        $self.ip += 1;
    }};
}

macro_rules! get_integers {
    ($self:ident, $( $name:ident ),*) => {
        $(let $name = $self.get_integer($name)?;)*
    };
}

macro_rules! get_strings {
    ($self:ident, $( $name:ident ),*) => {
        $(let $name = $self.get_str($name)?;)*
    };
}

pub struct VM<'a> {
    program: &'a ir::Program<'a>,
    config: &'a cfg::Config,
    ip: usize,
    vars: HashMap<ir::Identifier<'a>, u16>,
    vars_str: HashMap<ir::Identifier<'a>, &'a str>,
    call_stack: Vec<usize>,
    ctx: &'a mut dyn VMSys<'a>,
}

impl<'a> VM<'a> {
    pub fn new(
        program: &'a ir::Program<'a>,
        config: &'a cfg::Config,
        ctx: &'a mut dyn VMSys<'a>,
    ) -> Self {
        VM {
            program,
            config,
            ip: 0,
            vars: HashMap::new(),
            vars_str: HashMap::new(),
            call_stack: Vec::new(),
            ctx,
        }
    }

    fn get_integer(&mut self, i: ir::Integer<'a>) -> Result<u16, Error> {
        Ok(match i {
            ir::Integer::Literal(val) => val,
            ir::Integer::Variable(ident) => {
                if let Some(&val) = self.vars.get(&ident) {
                    val
                } else {
                    self.set_variable(ident, 0)?;
                    0
                }
            }
        })
    }

    fn get_str(&mut self, s: ir::Str<'a>) -> Result<&'a str, Error> {
        Ok(match s {
            ir::Str::Literal(val) => val,
            ir::Str::Variable(ident) => {
                if let Some(&val) = self.vars_str.get(&ident) {
                    val
                } else {
                    self.set_variable_str(ident, "")?;
                    ""
                }
            }
        })
    }

    fn set_variable(&mut self, ident: ir::Identifier<'a>, val: u16) -> Result<(), Error> {
        if self.config.pedantic && self.vars.len() >= 500 {
            Err(Error::ExcessVariablesError)
        } else {
            self.vars.insert(ident, val);
            Ok(())
        }
    }

    fn set_variable_str(&mut self, ident: ir::Identifier<'a>, val: &'a str) -> Result<(), Error> {
        if self.config.pedantic && self.vars_str.len() >= 200 {
            Err(Error::ExcessVariablesStrError)
        } else {
            self.vars_str.insert(ident, val);
            Ok(())
        }
    }

    fn goto_label(&mut self, label: ir::Identifier<'_>) -> Result<(), Error> {
        self.ip = *(self
            .program
            .labels
            .get(&label)
            .ok_or_else(|| Error::NonexistentLabelError)?);
        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, Error> {
        let cmd = &self.program.commands[self.ip];
        match *cmd {
            ir::Command::Beep => incr_ip!(self, self.ctx.beep()?),
            ir::Command::DrawArc {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2, x3, y3, x4, y4);
                self.ctx.draw_arc(x1, y1, x2, y2, x3, y3, x4, y4)?
            }),
            ir::Command::DrawBackground => incr_ip!(self, self.ctx.draw_background()?),
            ir::Command::DrawBitmap { x, y, filename } => incr_ip!(self, {
                get_integers!(self, x, y);
                get_strings!(self, filename);
                self.ctx.draw_bitmap(x, y, filename)?
            }),
            ir::Command::DrawChord {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2, x3, y3, x4, y4);
                self.ctx.draw_chord(x1, y1, x2, y2, x3, y3, x4, y4)?
            }),
            ir::Command::DrawEllipse { x1, y1, x2, y2 } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2);
                self.ctx.draw_ellipse(x1, y1, x2, y2)?
            }),
            ir::Command::DrawFlood { x, y, r, g, b } => incr_ip!(self, {
                get_integers!(self, x, y, r, g, b);
                self.ctx.draw_flood(x, y, r, g, b)?
            }),
            ir::Command::DrawLine { x1, y1, x2, y2 } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2);
                self.ctx.draw_line(x1, y1, x2, y2)?
            }),
            ir::Command::DrawNumber { x, y, n } => incr_ip!(self, {
                get_integers!(self, x, y, n);
                self.ctx.draw_number(x, y, n)?
            }),
            ir::Command::DrawPie {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2, x3, y3, x4, y4);
                self.ctx.draw_pie(x1, y1, x2, y2, x3, y3, x4, y4)?
            }),
            ir::Command::DrawRectangle { x1, y1, x2, y2 } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2);
                self.ctx.draw_rectangle(x1, y1, x2, y2)?
            }),
            ir::Command::DrawRoundRectangle {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2, x3, y3);
                self.ctx.draw_round_rectangle(x1, y1, x2, y2, x3, y3)?
            }),
            ir::Command::DrawSizedBitmap {
                x1,
                y1,
                x2,
                y2,
                filename,
            } => incr_ip!(self, {
                get_integers!(self, x1, y1, x2, y2);
                get_strings!(self, filename);
                self.ctx.draw_sized_bitmap(x1, y1, x2, y2, filename)?
            }),
            ir::Command::DrawText { x, y, text } => incr_ip!(self, {
                get_integers!(self, x, y);
                get_strings!(self, text);
                self.ctx.draw_text(x, y, text)?
            }),
            ir::Command::End => return Ok(false),
            ir::Command::Gosub(ident) => {
                self.call_stack.push(self.ip + 1);
                self.goto_label(ident)?
            }
            ir::Command::Return => {
                self.ip = self
                    .call_stack
                    .pop()
                    .ok_or_else(|| Error::CallStackExhaustedError)?;
            }
            ir::Command::Goto(ident) => self.goto_label(ident)?,
            ir::Command::If {
                i1,
                op,
                i2,
                goto_false,
            } => {
                self.ip = if op.cmp(self.get_integer(i1)?, self.get_integer(i2)?) {
                    self.ip + 1
                } else {
                    goto_false
                }
            }
            ir::Command::MessageBox {
                typ,
                default_button,
                icon,
                text,
                caption,
                button_pushed,
            } => {
                get_integers!(self, default_button);
                get_strings!(self, text, caption);
                let button_pushed_val =
                    self.ctx
                        .message_box(typ, default_button, icon, text, caption)?;
                incr_ip!(self, self.set_variable(button_pushed, button_pushed_val)?);
            }
            ir::Command::Run(command) => incr_ip!(self, {
                get_strings!(self, command);
                self.ctx.run(command)?
            }),
            ir::Command::Set { var, val } => incr_ip!(self, {
                let ident = match val {
                    ir::SetValue::Value(i) => self.get_integer(i)?,
                    ir::SetValue::Expression { i1, op, i2 } => op
                        .eval(self.get_integer(i1)?, self.get_integer(i2)?)
                        .ok_or_else(|| Error::MathOperationError)?,
                };
                self.set_variable(var, ident)?
            }),
            ir::Command::SetKeyboard(ref hashmap) => incr_ip!(self, {
                let params = hashmap
                    .iter()
                    .map(|(&key, &label)| {
                        Ok((
                            match key {
                                ir::Key::Virtual(integer) => Key::Virtual(
                                    self.get_integer(integer)?
                                        .try_into()
                                        .map_err(|_| Error::InvalidVirtualKeyError)?,
                                ),
                                ir::Key::Physical(physical) => Key::Physical({
                                    self.get_str(physical)?
                                        .try_into()
                                        .map_err(|_| Error::InvalidPhysicalKeyError)?
                                }),
                            },
                            label,
                        ))
                    })
                    .collect::<Result<HashMap<_, _>, Error>>()?;
                self.ctx.set_keyboard(params)?
            }),
            ir::Command::SetMenu(ref menu) => incr_ip!(self, {
                let menu = menu
                    .iter()
                    .map(|category| {
                        Ok(MenuCategory {
                            item: MenuItem {
                                name: self.get_str(category.item.name)?,
                                label: category.item.label,
                            },
                            members: category
                                .members
                                .iter()
                                .map(|member| {
                                    Ok(match member {
                                        ir::MenuMember::Item(item) => MenuMember::Item(MenuItem {
                                            name: self.get_str(item.name)?,
                                            label: item.label,
                                        }),
                                        ir::MenuMember::Separator => MenuMember::Separator,
                                    })
                                })
                                .collect::<Result<_, Error>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                self.ctx.set_menu(&menu)?
            }),
            ir::Command::SetMouse(ref params) => incr_ip!(self, {
                let params = &params
                    .iter()
                    .map(|param| {
                        Ok(MouseRegion {
                            x1: self.get_integer(param.x1)?,
                            y1: self.get_integer(param.y1)?,
                            x2: self.get_integer(param.x2)?,
                            y2: self.get_integer(param.y2)?,
                            callbacks: &param.callbacks,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                self.ctx.set_mouse(params)?
            }),
            ir::Command::SetWaitMode(mode) => incr_ip!(self, self.ctx.set_wait_mode(mode)?),
            ir::Command::SetWindow(option) => incr_ip!(self, self.ctx.set_window(option)?),
            ir::Command::UseBackground { option, r, g, b } => incr_ip!(self, {
                get_integers!(self, r, g, b);
                self.ctx.use_background(option, r, g, b)?
            }),
            ir::Command::UseBrush { option, r, g, b } => incr_ip!(self, {
                get_integers!(self, r, g, b);
                self.ctx.use_brush(option, r, g, b)?
            }),
            ir::Command::UseCaption(text) => incr_ip!(self, {
                get_strings!(self, text);
                self.ctx.use_caption(text)?
            }),
            ir::Command::UseCoordinates(coordinates) => {
                incr_ip!(self, self.ctx.use_coordinates(coordinates)?);
            }
            ir::Command::UseFont {
                name,
                width,
                height,
                bold,
                italic,
                underline,
                r,
                g,
                b,
            } => incr_ip!(self, {
                get_integers!(self, width, height, r, g, b);
                get_strings!(self, name);
                self.ctx
                    .use_font(name, width, height, bold, italic, underline, r, g, b)?
            }),
            ir::Command::UsePen {
                option,
                width,
                r,
                g,
                b,
            } => incr_ip!(self, {
                get_integers!(self, width, r, g, b);
                self.ctx.use_pen(option, width, r, g, b)?
            }),
            ir::Command::WaitInput(milliseconds) => {
                let milliseconds = if let Some(i) = milliseconds {
                    Some(self.get_integer(i)?)
                } else {
                    None
                };
                if let Some(input) = self.ctx.wait_input(milliseconds)? {
                    match input {
                        Input::End => return Ok(false),
                        Input::Goto(label) => self.goto_label(label)?,
                        Input::Mouse { callbacks, x, y } => {
                            self.set_variable(callbacks.x, x)?;
                            self.set_variable(callbacks.y, y)?;
                            self.goto_label(callbacks.label)?;
                        }
                    };
                } else {
                    self.ip += 1;
                }
            }
        };
        Ok(true)
    }

    pub fn run(&'a mut self) -> Result<(), Error> {
        loop {
            let step_result = self.step()?;

            if !step_result {
                break;
            }
        }
        Ok(())
    }
}
