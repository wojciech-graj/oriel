use std::collections::HashMap;

use crate::parse;

trait VMCtx {
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
        typ: parse::MessageBoxType,
        default_button: u16,
        icon: parse::MessageBoxIcon,
        text: &str,
        caption: &str,
    ) -> u16;
    fn run(&mut self, command: &str);
    fn set_keyboard(&mut self); // TODO
    fn set_menu(&mut self); // TODO
    fn set_mouse(&mut self); // TODO
    fn set_wait_mode(&mut self, mode: parse::WaitMode);
    fn set_window(&mut self, option: parse::SetWindowOption);
    fn use_background(&mut self, option: parse::UseBackgroundOption, r: u16, g: u16, g: u16);
    fn use_brush(&mut self, option: parse::UseBrushOption, r: u16, g: u16, b: u16);
    fn use_caption(&mut self, text: &str);
    fn use_coordinates(&mut self, option: parse::UseCoordinatesOption); // TODO: maybe?
    fn use_font(
        &mut self,
        name: &str,
        width: u16,
        height: u16,
        bold: parse::UseFontBold,
        italic: parse::UseFontItalic,
        underline: parse::UseFontUnderline,
        r: u16,
        g: u16,
        b: u16,
    );
    fn use_pen(&mut self, option: parse::UsePenOption, width: u16, r: u16, g: u16, b: u16);
    fn wait_input(&mut self, milliseconds: Option<u16>);
}

pub struct VM<'a> {
    program: parse::Program<'a>,
    ip: usize,
    vars: HashMap<parse::Identifier<'a>, u16>,
    ctx: &'a mut dyn VMCtx,
}

impl<'a> VM<'a> {
    fn new(program: parse::Program<'a>, ctx: &'a mut dyn VMCtx) -> Self {
        VM {
            program,
            ip: 0,
            vars: HashMap::new(),
            ctx,
        }
    }

    fn step(&mut self) {}

    fn run(&mut self) {}
}
