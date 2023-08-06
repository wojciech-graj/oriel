use std::cell::RefCell;
use std::f64::consts::PI;
use std::process;
use std::rc::Rc;
use std::time;

use gtk;
use gtk::cairo;
use gtk::gdk::prelude::*;
use gtk::gdk_pixbuf;
use gtk::prelude::*;

use crate::ir;
use crate::vm::*;

struct DrawState {
    text_face: cairo::FontFace,
    text_matrix: cairo::Matrix,
    text_underline: crate::ir::FontUnderline,
    text_rgb: (f64, f64, f64),

    pen_type: ir::PenType,
    pen_width: f64,
    pen_rgb: (f64, f64, f64),

    background_transparency: ir::BackgroundTransparency,
    background_rgb: (f64, f64, f64),

    brush_typ: ir::BrushType,
    brush_rgb: (f64, f64, f64),
}

impl DrawState {
    fn new() -> Self {
        DrawState {
            text_face: {
                let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, 0, 0).unwrap();
                let cr = cairo::Context::new(&surface).unwrap();
                cr.font_face()
            },
            text_matrix: cairo::Matrix::identity(),
            text_underline: ir::FontUnderline::NoUnderline,
            text_rgb: (0., 0., 0.),

            pen_type: ir::PenType::Solid,
            pen_width: 1.,
            pen_rgb: (0., 0., 0.),

            background_transparency: ir::BackgroundTransparency::Transparent,
            background_rgb: (1., 1., 1.),

            brush_typ: ir::BrushType::Null,
            brush_rgb: (0., 0., 0.),
        }
    }

    fn cr_text(&self, surface: &cairo::Surface) -> cairo::Context {
        let cr = cairo::Context::new(surface).unwrap();
        let (r, g, b) = self.text_rgb;
        cr.set_source_rgb(r, g, b);
        cr.set_font_matrix(self.text_matrix);
        cr
    }

    fn cr_pen(&self, surface: &cairo::Surface) -> cairo::Context {
        let cr = cairo::Context::new(surface).unwrap();
        let (r, g, b) = self.pen_rgb;
        cr.set_dash(
            match self.pen_type {
                ir::PenType::Solid => &[],
                ir::PenType::Null => &[0.],
                ir::PenType::Dash => &[24., 8.],
                ir::PenType::Dot => &[4.],
                ir::PenType::DashDot => &[12., 6., 3., 6.],
                ir::PenType::DashDotDot => &[12., 3., 3., 3., 3., 3.],
            },
            0.,
        );
        cr.set_line_width(self.pen_width);
        cr.set_source_rgb(r, g, b);
        cr
    }

    fn cr_background(&self, surface: &cairo::Surface) -> cairo::Context {
        let cr = cairo::Context::new(surface).unwrap();
        let (r, g, b) = self.background_rgb;
        cr.set_line_width(self.pen_width);
        cr.set_source_rgb(r, g, b);
        cr
    }

    fn cr_brush(&self, surface: &cairo::Surface) -> cairo::Context {
        let cr = cairo::Context::new(surface).unwrap();
        let (r, g, b) = self.brush_rgb;
        let pattern = cairo::SurfacePattern::create(match self.brush_typ {
            ir::BrushType::Solid => {
                let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, 1, 1).unwrap();
                let cr = cairo::Context::new(&surface).unwrap();
                cr.set_source_rgb(r, g, b);
                cr.paint().ok();
                surface
            }
            ir::BrushType::DiagonalUp => {
                let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, 8, 8).unwrap();
                let cr = cairo::Context::new(&surface).unwrap();
                let (bkg_r, bkg_g, bkg_b) = self.background_rgb;
                cr.set_antialias(cairo::Antialias::None);
                cr.set_source_rgb(bkg_r, bkg_g, bkg_b);
                cr.paint().ok();
                cr.set_source_rgb(r, g, b);
                cr.move_to(0.5, 8.);
                cr.line_to(8., 0.5);
                cr.rectangle(0., 0., 0.5, 0.5);
                cr.stroke().ok();
                surface
            }
            ir::BrushType::DiagonalDown => todo!(),
            ir::BrushType::DiagonalCross => todo!(),
            ir::BrushType::Horizontal => todo!(),
            ir::BrushType::Vertical => todo!(),
            ir::BrushType::Cross => todo!(),
            ir::BrushType::Null => {
                let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1).unwrap();
                let cr = cairo::Context::new(&surface).unwrap();
                cr.set_source_rgba(0., 0., 0., 0.);
                cr.paint().ok();
                surface
            }
        });
        pattern.set_extend(cairo::Extend::Repeat);
        cr.set_source(pattern).ok();
        cr
    }
}

struct DrawCtx {
    surface: cairo::ImageSurface,
    cr_text_: Option<cairo::Context>,
    cr_pen_: Option<cairo::Context>,
    cr_background_: Option<cairo::Context>,
    cr_brush_: Option<cairo::Context>,

    draw_state: DrawState,

    scale: f64,
}

impl DrawCtx {
    fn new() -> Self {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1).unwrap();
        let draw_state = DrawState::new();

        DrawCtx {
            surface,
            cr_text_: None,
            cr_pen_: None,
            cr_background_: None,
            cr_brush_: None,

            draw_state,

            scale: 1.,
        }
    }

    fn cr_text(&mut self) -> &cairo::Context {
        match self.cr_text_ {
            Some(ref cr_text) => cr_text,
            None => {
                let cr = self.draw_state.cr_text(&self.surface);
                self.cr_text_ = Some(cr);
                self.cr_text_.as_ref().unwrap()
            }
        }
    }

    fn cr_pen(&mut self) -> &cairo::Context {
        match self.cr_pen_ {
            Some(ref cr_pen) => cr_pen,
            None => {
                let cr = self.draw_state.cr_pen(&self.surface);
                self.cr_pen_ = Some(cr);
                self.cr_pen_.as_ref().unwrap()
            }
        }
    }

    fn cr_background(&mut self) -> &cairo::Context {
        match self.cr_background_ {
            Some(ref cr_background) => cr_background,
            None => {
                let cr = self.draw_state.cr_background(&self.surface);
                self.cr_background_ = Some(cr);
                self.cr_background_.as_ref().unwrap()
            }
        }
    }

    fn cr_brush(&mut self) -> &cairo::Context {
        match self.cr_brush_ {
            Some(ref cr_brush) => cr_brush,
            None => {
                let cr = self.draw_state.cr_brush(&self.surface);
                self.cr_brush_ = Some(cr);
                self.cr_brush_.as_ref().unwrap()
            }
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.surface = {
            let new = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
            let cr = cairo::Context::new(&new).unwrap();
            cr.set_source_surface(&self.surface, 0., 0.).ok();
            cr.paint().ok();
            new
        };
        self.cr_text_ = None;
        self.cr_pen_ = None;
        self.cr_background_ = None;
        self.cr_brush_ = None;
    }

    fn scaled(&self, x: u16) -> f64 {
        (x as f64) * self.scale
    }
}

pub struct VMSysGtk {
    window: gtk::Window,
    menu_bar: gtk::MenuBar,
    draw_ctx: Rc<RefCell<DrawCtx>>,
    wait_mode: ir::WaitMode,
}

impl VMSysGtk {
    pub fn new(filename: &str) -> Self {
        gtk::init().expect("Failed to initialize GTK.");

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_default_size(640, 480);
        window.set_resizable(false);
        window.set_title(format!("Oriel - {}", filename).as_str());

        let mainbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
        window.add(&mainbox);

        let menu_bar = {
            let menu_bar = gtk::MenuBar::new();
            let help = {
                let help = gtk::MenuItem::with_mnemonic("_Help");
                help.set_right_justified(true);
                help.connect_activate(|_| {
                    // TODO
                    let about = gtk::AboutDialog::new();
                    about.show_all();
                });
                help
            };
            menu_bar.append(&help);
            menu_bar
        };
        mainbox.pack_start(&menu_bar, false, true, 0);

        let drawing_area = gtk::DrawingArea::new();
        mainbox.pack_start(&drawing_area, true, true, 0);

        let draw_ctx = { Rc::new(RefCell::new(DrawCtx::new())) };

        let draw_ctx_clone = draw_ctx.clone();
        drawing_area.connect_draw(move |_, ctx| {
            let draw_ctx = draw_ctx_clone.borrow();
            ctx.set_source_surface(draw_ctx.surface.as_ref(), 0., 0.)
                .ok();
            ctx.paint().ok();
            Inhibit(false)
        });

        let draw_ctx_clone = draw_ctx.clone();
        drawing_area.connect_size_allocate(move |_, rect| {
            draw_ctx_clone
                .borrow_mut()
                .resize(rect.width(), rect.height());
        });

        window.show_all();

        let mut sys = VMSysGtk {
            window,
            menu_bar,
            draw_ctx,
            wait_mode: ir::WaitMode::Null,
        };

        sys.use_coordinates(ir::Coordinates::Metric);

        sys
    }

    fn line_exec(&self, brush: bool, op: impl Fn(&cairo::Context)) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        if brush {
            match draw_ctx.draw_state.brush_typ {
                ir::BrushType::Solid
                | ir::BrushType::DiagonalUp
                | ir::BrushType::DiagonalDown
                | ir::BrushType::DiagonalCross
                | ir::BrushType::Horizontal
                | ir::BrushType::Vertical
                | ir::BrushType::Cross => {
                    op(draw_ctx.cr_brush());
                }
                ir::BrushType::Null => {}
            }
        }
        match draw_ctx.draw_state.background_transparency {
            ir::BackgroundTransparency::Opaque => {
                op(draw_ctx.cr_background());
            }
            ir::BackgroundTransparency::Transparent => {}
        }
        match draw_ctx.draw_state.pen_type {
            ir::PenType::Solid
            | ir::PenType::Dash
            | ir::PenType::Dot
            | ir::PenType::DashDot
            | ir::PenType::DashDotDot => {
                op(draw_ctx.cr_pen());
            }
            ir::PenType::Null => {}
        }
    }

    fn arc_path(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        theta1: f64,
        theta2: f64,
        mv: bool,
        brush: bool,
    ) -> (f64, f64) {
        const DTHETA: f64 = -0.1;

        let sclx = (x2 - x1) / 2.;
        let scly = (y2 - y1) / 2.;
        let cx = ((x2 + x1) as f64) / 2.;
        let cy = ((y2 + y1) as f64) / 2.;

        let startx = cx + sclx * theta1.cos();
        let starty = cy + scly * theta1.sin();
        let endx = cx + sclx * theta2.cos();
        let endy = cy + scly * theta2.sin();

        if mv {
            self.line_exec(brush, |ctx| {
                ctx.move_to(startx, starty);
            });
        }
        let mut theta = theta1;
        while theta > theta2 {
            self.line_exec(brush, |ctx| {
                ctx.line_to(cx + sclx * theta.cos(), cy + scly * theta.sin());
            });
            theta += DTHETA;
        }
        self.line_exec(brush, |ctx| {
            ctx.line_to(endx, endy);
        });

        (startx, starty)
    }

    fn draw(&self) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();
        draw_ctx.cr_brush().fill().ok();
        draw_ctx.cr_background().stroke().ok();
        draw_ctx.cr_pen().stroke().ok();
    }
}

fn filename_conv(filename: &str) -> &str {
    match filename {
        "C:\\WINDOWS\\BOXES.BMP" => "res/BOXES.BMP",
        "C:\\WINDOWS\\CHESS.BMP" => "res/CHESS.BMP",
        "C:\\WINDOWS\\PAPER.BMP" => "res/PAPER.BMP",
        "C:\\WINDOWS\\PARTY.BMP" => "res/PARTY.BMP",
        "C:\\WINDOWS\\PYRAMID.BMP" => "res/PYRAMID.BMP",
        "C:\\WINDOWS\\RIBBONS.BMP" => "res/RIBBONS.BMP",
        "C:\\WINDOWS\\WEAVE.BMP" => "res/WEAVE.BMP",
        filename => filename,
    }
}

fn command_conv(command: &str) -> &str {
    match command {
        "NOTEPAD.EXE" => "mousepad",
        "CALC.EXE" => "libreoffice --calc",
        "WRITE.EXE" => "libreoffice --writer",
        "C:\\COMMAND.COM" => "xterm",
        command => command,
    }
}

impl<'a> VMSys<'a> for VMSysGtk {
    fn beep(&mut self) {
        self.window.window().unwrap().beep();
    }

    fn draw_arc(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16, x4: u16, y4: u16) {
        let (x1, y1, x2, y2, x3, y3, x4, y4) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
                draw_ctx.scaled(x3),
                draw_ctx.scaled(y3),
                draw_ctx.scaled(x4),
                draw_ctx.scaled(y4),
            )
        };

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx);
        let theta2 = (y4 - cy).atan2(x4 - cx);

        self.arc_path(x1, y1, x2, y2, theta1, theta2, true, false);
        self.line_exec(false, |ctx| {
            ctx.stroke().ok();
        });
    }

    fn draw_background(&mut self) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        draw_ctx.cr_background().paint().ok();
    }

    fn draw_bitmap(&mut self, x: u16, y: u16, filename: &str) {
        let (x, y) = {
            let draw_ctx = self.draw_ctx.borrow();

            (draw_ctx.scaled(x), draw_ctx.scaled(y))
        };

        let filename = filename_conv(filename);

        let pixbuf = gdk_pixbuf::Pixbuf::from_file(filename).unwrap();
        let surface = pixbuf
            .create_surface(1, self.window.window().as_ref())
            .unwrap();

        let draw_ctx = self.draw_ctx.borrow();
        let cr = cairo::Context::new(draw_ctx.surface.as_ref()).unwrap();
        cr.set_source_surface(&surface, x, y).ok();
        cr.paint().ok();
    }

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
    ) {
        let (x1, y1, x2, y2, x3, y3, x4, y4) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
                draw_ctx.scaled(x3),
                draw_ctx.scaled(y3),
                draw_ctx.scaled(x4),
                draw_ctx.scaled(y4),
            )
        };

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx); //TODO: scale before atan
        let theta2 = (y4 - cy).atan2(x4 - cx);

        let pts = self.arc_path(x1, y1, x2, y2, theta1, theta2, true, true);
        self.line_exec(true, |ctx| {
            ctx.line_to(pts.0, pts.1);
        });

        self.draw();
    }

    fn draw_ellipse(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let (x1, y1, x2, y2) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
            )
        };

        self.arc_path(x1, y1, x2, y2, 6.28, 0.0, true, true);
        self.draw();
    }

    fn draw_flood(&mut self, x: u16, y: u16, r: u16, g: u16, b: u16) {
        //TODO
    }

    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let (x1, y1, x2, y2) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
            )
        };

        self.line_exec(false, |ctx| {
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
        });
        self.line_exec(false, |ctx| {
            ctx.stroke().ok();
        });
    }

    fn draw_number(&mut self, x: u16, y: u16, n: u16) {
        self.draw_text(x, y, n.to_string().as_str());
    }

    fn draw_pie(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16, x4: u16, y4: u16) {
        let (x1, y1, x2, y2, x3, y3, x4, y4) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
                draw_ctx.scaled(x3),
                draw_ctx.scaled(y3),
                draw_ctx.scaled(x4),
                draw_ctx.scaled(y4),
            )
        };

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx);
        let theta2 = (y4 - cy).atan2(x4 - cx);

        let pts = self.arc_path(x1, y1, x2, y2, theta1, theta2, true, true);
        self.line_exec(true, |ctx| {
            ctx.line_to(cx, cy);
            ctx.line_to(pts.0, pts.1);
        });
        self.draw();
    }

    fn draw_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let (x1, y1, x2, y2) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
            )
        };

        self.line_exec(true, |ctx| {
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y1);
            ctx.line_to(x2, y2);
            ctx.line_to(x1, y2);
            ctx.line_to(x1, y1);
        });
        self.draw();
    }

    fn draw_round_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16) {
        let (x1, y1, x2, y2, x3, y3) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
                draw_ctx.scaled(x3),
                draw_ctx.scaled(y3),
            )
        };

        self.arc_path(x1, y1, x1 + x3, y1 + y3, PI * 1.5, PI, false, true);
        self.line_exec(true, |ctx| {
            ctx.line_to(x1, y2 - y3 / 2.);
        });
        self.arc_path(x1, y2 - y3, x1 + x3, y2, PI, PI * 0.5, false, true);
        self.line_exec(true, |ctx| {
            ctx.line_to(x2 - x3 / 2., y2);
        });
        self.arc_path(x2 - x3, y2 - y3, x2, y2, PI * 0.5, 0., false, true);
        self.line_exec(true, |ctx| {
            ctx.line_to(x2, y1 + y3 / 2.);
        });
        self.arc_path(x2 - x3, y1, x2, y1 + y3, 0., PI * -0.5, false, true);
        self.line_exec(true, |ctx| {
            ctx.line_to(x1 + x3 / 2., y1);
        });

        self.draw();
    }

    fn draw_sized_bitmap(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, filename: &str) {
        let (x1, y1, x2, y2) = {
            let draw_ctx = self.draw_ctx.borrow();

            (
                draw_ctx.scaled(x1),
                draw_ctx.scaled(y1),
                draw_ctx.scaled(x2),
                draw_ctx.scaled(y2),
            )
        };
        let filename = filename_conv(filename);

        let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_size(
            filename,
            (x1 - x2).abs() as i32,
            (y1 - y2).abs() as i32,
        )
        .unwrap();
        let surface = pixbuf
            .create_surface(1, self.window.window().as_ref())
            .unwrap();

        let draw_ctx = self.draw_ctx.borrow();
        let cr = cairo::Context::new(draw_ctx.surface.as_ref()).unwrap();
        cr.scale(
            if x1 < x2 { 1. } else { -1. },
            if y1 < y2 { 1. } else { -1. },
        );
        cr.translate(
            if x1 < x2 {
                0.
            } else {
                (-pixbuf.width()).into()
            },
            if y1 < y2 {
                0.
            } else {
                (-pixbuf.height()).into()
            },
        );
        cr.set_source_surface(&surface, x1.min(x2), y1.min(y2)).ok();
        cr.paint().ok();
    }

    fn draw_text(&mut self, x: u16, y: u16, text: &str) {
        let (x, y) = {
            let draw_ctx = self.draw_ctx.borrow();

            (draw_ctx.scaled(x), draw_ctx.scaled(y))
        };

        let mut draw_ctx = self.draw_ctx.borrow_mut();

        if let ir::BackgroundTransparency::Opaque = draw_ctx.draw_state.background_transparency {
            let text_extents = draw_ctx.cr_text().text_extents(text).unwrap();
            let font_extents = draw_ctx.cr_text().font_extents().unwrap();
            draw_ctx.cr_background().rectangle(
                x,
                y - font_extents.ascent(),
                text_extents.width(),
                font_extents.height(),
            );
            draw_ctx.cr_background().fill().ok();
        }

        draw_ctx.cr_text().move_to(x, y);
        draw_ctx.cr_text().show_text(text).ok();
    }

    fn message_box(
        &mut self,
        typ: crate::ir::MessageBoxType,
        default_button: u16, //TODO
        icon: crate::ir::MessageBoxIcon,
        text: &str,
        caption: &str,
    ) -> u16 {
        let dialog = gtk::MessageDialog::new(
            Some(&self.window),
            gtk::DialogFlags::DESTROY_WITH_PARENT,
            match icon {
                ir::MessageBoxIcon::Information => gtk::MessageType::Info,
                ir::MessageBoxIcon::Exclamation => gtk::MessageType::Warning,
                ir::MessageBoxIcon::Question => gtk::MessageType::Question,
                ir::MessageBoxIcon::Stop => gtk::MessageType::Error,
                ir::MessageBoxIcon::NoIcon => gtk::MessageType::Other,
            },
            gtk::ButtonsType::None,
            text,
        );
        dialog.set_title(caption);
        dialog.add_buttons(match typ {
            ir::MessageBoxType::Ok => &[("Ok", gtk::ResponseType::Other(1))],
            ir::MessageBoxType::OkCancel => &[
                ("Ok", gtk::ResponseType::Other(1)),
                ("Cancel", gtk::ResponseType::Other(2)),
            ],
            ir::MessageBoxType::YesNo => &[
                ("Yes", gtk::ResponseType::Other(1)),
                ("No", gtk::ResponseType::Other(2)),
            ],
            ir::MessageBoxType::YesNoCancel => &[
                ("Yes", gtk::ResponseType::Other(1)),
                ("No", gtk::ResponseType::Other(2)),
                ("Cancel", gtk::ResponseType::Other(3)),
            ],
        });

        let response = dialog.run();
        dialog.close();
        while gtk::events_pending() {
            gtk::main_iteration();
        }

        if let gtk::ResponseType::Other(x) = response {
            x
        } else {
            default_button
        }
    }

    fn run(&mut self, command: &str) {
        let command = command_conv(command);

        process::Command::new(command).spawn().ok();
    }

    fn set_keyboard(&mut self) {
        todo!()
    }

    fn set_menu(&mut self, menu: Vec<ir::MenuItem<'a>>) {
        todo!()
    }

    fn set_mouse(&mut self) {
        todo!()
    }

    fn set_wait_mode(&mut self, mode: crate::ir::WaitMode) {
        self.wait_mode = mode;
    }

    fn set_window(&mut self, option: crate::ir::SetWindowOption) {
        match option {
            ir::SetWindowOption::Maximize => self.window.maximize(),
            ir::SetWindowOption::Minimize => self.window.iconify(),
            ir::SetWindowOption::Restore => {
                self.window.unmaximize();
                self.window.deiconify();
            }
        }
    }

    fn use_background(
        &mut self,
        option: crate::ir::BackgroundTransparency,
        r: u16,
        g: u16,
        b: u16,
    ) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();
        draw_ctx.draw_state.background_transparency = option;
        draw_ctx.draw_state.background_rgb =
            ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_background_ = None;
        draw_ctx.cr_brush_ = None;
    }

    fn use_brush(&mut self, option: crate::ir::BrushType, r: u16, g: u16, b: u16) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();
        draw_ctx.draw_state.brush_typ = option;
        draw_ctx.draw_state.brush_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_brush_ = None;
    }

    fn use_caption(&mut self, text: &str) {
        self.window.set_title(text);
    }

    fn use_coordinates(&mut self, option: crate::ir::Coordinates) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        draw_ctx.scale = match option {
            ir::Coordinates::Pixel => 1.,
            ir::Coordinates::Metric => {
                let window_gdk = self.window.window().unwrap();
                let monitor = window_gdk.display().monitor_at_window(&window_gdk).unwrap();
                (monitor.geometry().width() as f64) / (monitor.width_mm() as f64)
            }
        };
    }

    fn use_font(
        &mut self,
        name: &str,
        width: u16,
        height: u16,
        bold: crate::ir::FontWeight,
        italic: crate::ir::FontSlant,
        underline: crate::ir::FontUnderline,
        r: u16,
        g: u16,
        b: u16,
    ) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        draw_ctx.draw_state.text_underline = underline;

        let font_face = cairo::FontFace::toy_create(
            name,
            match italic {
                ir::FontSlant::Italic => cairo::FontSlant::Italic,
                ir::FontSlant::NoItalic => cairo::FontSlant::Normal,
            },
            match bold {
                ir::FontWeight::Bold => cairo::FontWeight::Bold,
                ir::FontWeight::NoBold => cairo::FontWeight::Normal,
            },
        )
        .unwrap();

        let mut matrix = cairo::Matrix::identity();
        draw_ctx.draw_state.text_face = font_face;
        draw_ctx.draw_state.text_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.draw_state.text_matrix = matrix;
        draw_ctx.cr_text_ = None;

        if width == 0 && height == 0 {
            return;
        }

        let extents = draw_ctx.cr_text().font_extents().unwrap();
        if width != 0 {
            matrix.set_xx(draw_ctx.scaled(width) / extents.max_x_advance());
        }
        if height != 0 {
            matrix.set_yy(draw_ctx.scaled(height) / extents.height());
        }
        draw_ctx.draw_state.text_matrix = matrix;
        draw_ctx.cr_text_ = None;
    }

    fn use_pen(&mut self, option: crate::ir::PenType, width: u16, r: u16, g: u16, b: u16) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        draw_ctx.draw_state.pen_type = option;
        draw_ctx.draw_state.pen_width = width.into();
        draw_ctx.draw_state.pen_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_pen_ = None;
        draw_ctx.cr_background_ = None;
    }

    fn wait_input(&mut self, milliseconds: Option<u16>) {
        self.window.queue_draw();
        match self.wait_mode {
            ir::WaitMode::Null => {
                if let Some(milliseconds) = milliseconds {
                    let milliseconds = if milliseconds == 0 { 1 } else { milliseconds };
                    let start = time::Instant::now();
                    while start.elapsed().as_millis() < milliseconds.into() {
                        while gtk::events_pending() {
                            gtk::main_iteration();
                        }
                    }
                } else {
                    //TODO
                    while self.window.is_visible() {
                        while gtk::events_pending() {
                            gtk::main_iteration();
                        }
                    }
                }
            }
            ir::WaitMode::Focus => {
                if let Some(_milliseconds) = milliseconds {
                    while !self.window.has_focus() {
                        while gtk::events_pending() {
                            gtk::main_iteration();
                        }
                    }
                }
            }
        }
    }
}
