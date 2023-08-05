use std::collections::HashMap;

use thiserror::Error;

use crate::ir::*;

impl LogicalOperator {
    fn cmp(&self, i1: u16, i2: u16) -> bool {
        match self {
            LogicalOperator::Equal => i1 == i2,
            LogicalOperator::Less => i1 < i2,
            LogicalOperator::Greater => i1 > i2,
            LogicalOperator::LEqual => i1 <= i2,
            LogicalOperator::GEqual => i1 >= i2,
            LogicalOperator::NEqual => i1 != i2,
        }
    }
}

impl MathOperator {
    fn eval(&self, i1: u16, i2: u16) -> Option<u16> {
        (match self {
            MathOperator::Add => u16::checked_add,
            MathOperator::Subtract => u16::checked_sub,
            MathOperator::Multiply => u16::checked_mul,
            MathOperator::Divide => u16::checked_div,
        })(i1, i2)
    }
}

pub trait VMSys<'a> {
    fn beep(&mut self);
    fn draw_arc(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16, x4: u16, y4: u16);
    fn draw_background(&mut self);
    fn draw_bitmap(&mut self, x: u16, y: u16, filename: &str);
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
    );
    fn draw_ellipse(&mut self, x1: u16, y1: u16, x2: u16, y2: u16);
    fn draw_flood(&mut self, x: u16, y: u16, r: u16, g: u16, b: u16);
    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16);
    fn draw_number(&mut self, x: u16, y: u16, n: u16);
    fn draw_pie(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16, x4: u16, y4: u16);
    fn draw_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16);
    fn draw_round_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16);
    fn draw_sized_bitmap(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, filename: &str);
    fn draw_text(&mut self, x: u16, y: u16, text: &str);
    fn message_box(
        &mut self,
        typ: MessageBoxType,
        default_button: u16,
        icon: MessageBoxIcon,
        text: &str,
        caption: &str,
    ) -> u16;
    fn run(&mut self, command: &str);
    fn set_keyboard(&mut self); // TODO
    fn set_menu(&mut self, menu: Vec<MenuItem<'a>>);
    fn set_mouse(&mut self); // TODO
    fn set_wait_mode(&mut self, mode: WaitMode);
    fn set_window(&mut self, option: SetWindowOption);
    fn use_background(&mut self, option: BackgroundTransparency, r: u16, g: u16, b: u16);
    fn use_brush(&mut self, option: BrushType, r: u16, g: u16, b: u16);
    fn use_caption(&mut self, text: &str);
    fn use_coordinates(&mut self, option: Coordinates);
    fn use_font(
        &mut self,
        name: &str,
        width: u16,
        height: u16,
        bold: FontWeight,
        italic: FontSlant,
        underline: FontUnderline,
        r: u16,
        g: u16,
        b: u16,
    );
    fn use_pen(&mut self, option: PenType, width: u16, r: u16, g: u16, b: u16);
    fn wait_input(&mut self, milliseconds: Option<u16>);
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("Attempted to use undeclared variable '{}'", (.0).0)]
    UndeclaredVariableError(Identifier<'a>),
    #[error("Call stack exhausted")]
    CallStackExhaustedError,
    #[error("Integer Under/Over-flow")]
    MathOperationError,
}

pub struct VM<'a> {
    program: &'a Program<'a>,
    ip: usize,
    vars: HashMap<Identifier<'a>, u16>,
    call_stack: Vec<usize>,
    ctx: &'a mut dyn VMSys<'a>,
}

macro_rules! integer_value {
    ($self:ident, $id:ident) => {{
        $self.get_integer($id).ok_or_else(|| {
            Error::UndeclaredVariableError(match $id {
                Integer::Variable(id) => id,
                Integer::Literal(_) => unreachable!(),
            })
        })?
    }};
}

macro_rules! incr_ip {
    ($self:ident, $e:expr) => {{
        $e;
        $self.ip += 1;
    }};
}

impl<'a> VM<'a> {
    pub fn new(program: &'a Program<'a>, ctx: &'a mut dyn VMSys<'a>) -> Self {
        VM {
            program,
            ip: 0,
            vars: HashMap::new(),
            call_stack: Vec::new(),
            ctx,
        }
    }

    fn get_integer(&self, i: Integer) -> Option<u16> {
        match i {
            Integer::Literal(val) => Some(val),
            Integer::Variable(ref ident) => self.vars.get(ident).cloned(),
        }
    }

    fn set_variable(&mut self, ident: Identifier<'a>, val: u16) {
        self.vars.insert(ident, val);
    }

    pub fn step(&mut self) -> Result<bool, Error<'a>> {
        let cmd = &self.program.commands[self.ip];
        match *cmd {
            Command::Beep => incr_ip!(self, self.ctx.beep()),
            Command::DrawArc {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            } => incr_ip!(
                self,
                self.ctx.draw_arc(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                    integer_value!(self, x3),
                    integer_value!(self, y3),
                    integer_value!(self, x4),
                    integer_value!(self, y4),
                )
            ),
            Command::DrawBackground => incr_ip!(self, self.ctx.draw_background()),
            Command::DrawBitmap { x, y, filename } => incr_ip!(
                self,
                self.ctx
                    .draw_bitmap(integer_value!(self, x), integer_value!(self, y), filename)
            ),
            Command::DrawChord {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            } => incr_ip!(
                self,
                self.ctx.draw_chord(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                    integer_value!(self, x3),
                    integer_value!(self, y3),
                    integer_value!(self, x4),
                    integer_value!(self, y4),
                )
            ),
            Command::DrawEllipse { x1, y1, x2, y2 } => incr_ip!(
                self,
                self.ctx.draw_ellipse(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                )
            ),
            Command::DrawFlood { x, y, r, g, b } => incr_ip!(
                self,
                self.ctx.draw_flood(
                    integer_value!(self, x),
                    integer_value!(self, y),
                    integer_value!(self, r),
                    integer_value!(self, g),
                    integer_value!(self, b),
                )
            ),
            Command::DrawLine { x1, y1, x2, y2 } => incr_ip!(
                self,
                self.ctx.draw_line(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                )
            ),
            Command::DrawNumber { x, y, n } => incr_ip!(
                self,
                self.ctx.draw_number(
                    integer_value!(self, x),
                    integer_value!(self, y),
                    integer_value!(self, n),
                )
            ),
            Command::DrawPie {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            } => incr_ip!(
                self,
                self.ctx.draw_pie(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                    integer_value!(self, x3),
                    integer_value!(self, y3),
                    integer_value!(self, x4),
                    integer_value!(self, y4),
                )
            ),
            Command::DrawRectangle { x1, y1, x2, y2 } => incr_ip!(
                self,
                self.ctx.draw_rectangle(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                )
            ),
            Command::DrawRoundRectangle {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => incr_ip!(
                self,
                self.ctx.draw_round_rectangle(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                    integer_value!(self, x3),
                    integer_value!(self, y3),
                )
            ),
            Command::DrawSizedBitmap {
                x1,
                y1,
                x2,
                y2,
                filename,
            } => incr_ip!(
                self,
                self.ctx.draw_sized_bitmap(
                    integer_value!(self, x1),
                    integer_value!(self, y1),
                    integer_value!(self, x2),
                    integer_value!(self, y2),
                    filename,
                )
            ),
            Command::DrawText { x, y, text } => incr_ip!(
                self,
                self.ctx
                    .draw_text(integer_value!(self, x), integer_value!(self, y), text)
            ),
            Command::End => return Ok(false),
            Command::Gosub(ref ident) => {
                self.call_stack.push(self.ip + 1);
                self.ip = *(self.program.labels.get(ident).unwrap());
            }
            Command::Return => {
                self.ip = self
                    .call_stack
                    .pop()
                    .ok_or_else(|| Error::CallStackExhaustedError)?
            }
            Command::Goto(ref ident) => self.ip = *(self.program.labels.get(ident).unwrap()),
            Command::If {
                i1,
                op,
                i2,
                goto_false,
            } => {
                self.ip = if !op.cmp(integer_value!(self, i1), integer_value!(self, i2)) {
                    goto_false
                } else {
                    self.ip + 1
                }
            }
            Command::MessageBox {
                typ,
                default_button,
                icon,
                text,
                caption,
                button_pushed,
            } => {
                let button_pushed_val = self.ctx.message_box(
                    typ,
                    integer_value!(self, default_button),
                    icon,
                    text,
                    caption,
                );
                incr_ip!(self, self.set_variable(button_pushed, button_pushed_val));
            }
            Command::Run(command) => incr_ip!(self, self.ctx.run(command)),
            Command::Set { var, i1, op, i2 } => incr_ip!(
                self,
                self.set_variable(
                    var,
                    op.eval(integer_value!(self, i1), integer_value!(self, i2))
                        .ok_or_else(|| Error::MathOperationError)?,
                )
            ),
            Command::SetKeyboard(_) => {} //TODO
            Command::SetMenu(_) => {}     //TODO
            Command::SetMouse(_) => {}    //TODO
            Command::SetWaitMode(mode) => incr_ip!(self, self.ctx.set_wait_mode(mode)),
            Command::SetWindow(option) => incr_ip!(self, self.ctx.set_window(option)),
            Command::UseBackground { option, r, g, b } => incr_ip!(
                self,
                self.ctx.use_background(
                    option,
                    integer_value!(self, r),
                    integer_value!(self, g),
                    integer_value!(self, b),
                )
            ),
            Command::UseBrush { option, r, g, b } => incr_ip!(
                self,
                self.ctx.use_brush(
                    option,
                    integer_value!(self, r),
                    integer_value!(self, g),
                    integer_value!(self, b),
                )
            ),
            Command::UseCaption(text) => incr_ip!(self, self.ctx.use_caption(text)),
            Command::UseCoordinates(coordinates) => {
                incr_ip!(self, self.ctx.use_coordinates(coordinates))
            }
            Command::UseFont {
                name,
                width,
                height,
                bold,
                italic,
                underline,
                r,
                g,
                b,
            } => incr_ip!(
                self,
                self.ctx.use_font(
                    name,
                    integer_value!(self, width),
                    integer_value!(self, height),
                    bold,
                    italic,
                    underline,
                    integer_value!(self, r),
                    integer_value!(self, g),
                    integer_value!(self, b),
                )
            ),
            Command::UsePen {
                option,
                width,
                r,
                g,
                b,
            } => incr_ip!(
                self,
                self.ctx.use_pen(
                    option,
                    integer_value!(self, width),
                    integer_value!(self, r),
                    integer_value!(self, g),
                    integer_value!(self, b),
                )
            ),
            Command::WaitInput(milliseconds) => incr_ip!(
                self,
                self.ctx.wait_input(if let Some(i) = milliseconds {
                    Some(integer_value!(self, i))
                } else {
                    None
                })
            ),
        };
        Ok(true)
    }

    pub fn run(&'a mut self) -> Result<(), Error<'a>> {
        loop {
            let step_result = self.step()?;

            if !step_result {
                break;
            }
        }
        Ok(())
    }
}
