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

struct DrawCtx {
    surface: cairo::ImageSurface,
    cr_text: cairo::Context,
    cr_pen: cairo::Context,
    cr_background: cairo::Context,
    cr_brush: cairo::Context,
}

pub struct VMSysGtk {
    window: gtk::Window,
    menu_bar: gtk::MenuBar,
    draw_ctx: Rc<RefCell<DrawCtx>>,
    underline: crate::ir::FontUnderline,
    background: crate::ir::BackgroundTransparency,
    pen_type: ir::PenType,
    brush_type: ir::BrushType,
    wait_mode: ir::WaitMode,
    scale: f64,
}

impl VMSysGtk {
    pub fn new() -> Self {
        gtk::init().expect("Failed to initialize GTK.");

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_default_size(640, 480);
        window.set_resizable(false);

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

        let draw_ctx = {
            let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1).unwrap();
            let cr_text = cairo::Context::new(&surface).unwrap();
            let cr_pen = cairo::Context::new(&surface).unwrap();
            let cr_background = cairo::Context::new(&surface).unwrap();
            let cr_brush = cairo::Context::new(&surface).unwrap();
            Rc::new(RefCell::new(DrawCtx {
                surface,
                cr_text,
                cr_pen,
                cr_background,
                cr_brush,
            }))
        };

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
            let mut draw_ctx_mut = draw_ctx_clone.borrow_mut();
            *draw_ctx_mut = {
                let surface =
                    cairo::ImageSurface::create(cairo::Format::ARgb32, rect.width(), rect.height())
                        .unwrap();
                let cr = cairo::Context::new(&surface).unwrap();
                cr.set_source_surface(draw_ctx_mut.surface.as_ref(), 0., 0.)
                    .ok();
                cr.paint().ok();

                // TODO: dup ctx parms
                let cr_text = cairo::Context::new(&surface).unwrap();
                let cr_pen = cairo::Context::new(&surface).unwrap();
                let cr_background = cairo::Context::new(&surface).unwrap();
                let cr_brush = cairo::Context::new(&surface).unwrap();
                DrawCtx {
                    surface,
                    cr_text,
                    cr_pen,
                    cr_background,
                    cr_brush,
                }
            }
        });

        window.show_all();

        let mut sys = VMSysGtk {
            window,
            menu_bar,
            draw_ctx,
            underline: ir::FontUnderline::NoUnderline,
            background: ir::BackgroundTransparency::Transparent,
            pen_type: ir::PenType::Solid,
            brush_type: ir::BrushType::Null,
            wait_mode: ir::WaitMode::Null,
            scale: 1.,
        };

        sys.use_coordinates(ir::Coordinates::Metric);
        sys.use_brush(ir::BrushType::Null, 0, 0, 0);

        sys
    }

    fn line_exec(&self, brush: bool, op: impl Fn(&cairo::Context)) {
        let draw_ctx = self.draw_ctx.borrow();

        if brush {
            match self.brush_type {
                ir::BrushType::Solid
                | ir::BrushType::DiagonalUp
                | ir::BrushType::DiagonalDown
                | ir::BrushType::DiagonalCross
                | ir::BrushType::Horizontal
                | ir::BrushType::Vertical
                | ir::BrushType::Cross => {
                    op(&draw_ctx.cr_brush);
                }
                ir::BrushType::Null => {}
            }
        }
        match self.background {
            ir::BackgroundTransparency::Opaque => {
                op(&draw_ctx.cr_background);
            }
            ir::BackgroundTransparency::Transparent => {}
        }
        match self.pen_type {
            ir::PenType::Solid
            | ir::PenType::Dash
            | ir::PenType::Dot
            | ir::PenType::DashDot
            | ir::PenType::DashDotDot => {
                op(&draw_ctx.cr_pen);
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
        let draw_ctx = self.draw_ctx.borrow();
        //draw_ctx.cr_brush.clip();
        draw_ctx.cr_brush.fill().ok();
        //draw_ctx.cr_brush.reset_clip();
        draw_ctx.cr_background.stroke().ok();
        draw_ctx.cr_pen.stroke().ok();
    }

    fn scaled(&self, x: u16) -> f64 {
        (x as f64) * self.scale
    }
}

impl<'a> VMSys<'a> for VMSysGtk {
    fn beep(&mut self) {
        self.window.window().unwrap().beep();
    }

    fn draw_arc(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16, x4: u16, y4: u16) {
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);
        let x3 = self.scaled(x3);
        let y3 = self.scaled(y3);
        let x4 = self.scaled(x4);
        let y4 = self.scaled(y4);

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
        let draw_ctx = self.draw_ctx.borrow();

        draw_ctx.cr_background.paint().ok();
    }

    fn draw_bitmap(&mut self, x: u16, y: u16, filename: &str) {
        let x = self.scaled(x);
        let y = self.scaled(y);

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
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);
        let x3 = self.scaled(x3);
        let y3 = self.scaled(y3);
        let x4 = self.scaled(x4);
        let y4 = self.scaled(y4);

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx);
        let theta2 = (y4 - cy).atan2(x4 - cx);

        let pts = self.arc_path(x1, y1, x2, y2, theta1, theta2, true, true);
        self.line_exec(true, |ctx| {
            ctx.line_to(pts.0, pts.1);
        });

        self.draw();
    }

    fn draw_ellipse(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);

        self.arc_path(x1, y1, x2, y2, 6.28, 0.0, true, true);
        self.draw();
    }

    fn draw_flood(&mut self, x: u16, y: u16, r: u16, g: u16, b: u16) {
        todo!()
    }

    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);

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
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);
        let x3 = self.scaled(x3);
        let y3 = self.scaled(y3);
        let x4 = self.scaled(x4);
        let y4 = self.scaled(y4);

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
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);

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
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);
        let x3 = self.scaled(x3);
        let y3 = self.scaled(y3);

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
        let x1 = self.scaled(x1);
        let y1 = self.scaled(y1);
        let x2 = self.scaled(x2);
        let y2 = self.scaled(y2);

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
        let x = self.scaled(x);
        let y = self.scaled(y);

        let draw_ctx = self.draw_ctx.borrow();

        if let ir::BackgroundTransparency::Opaque = self.background {
            let text_extents = draw_ctx.cr_text.text_extents(text).unwrap();
            let font_extents = draw_ctx.cr_text.font_extents().unwrap();
            draw_ctx.cr_background.rectangle(
                x,
                y - font_extents.ascent(),
                text_extents.width(),
                font_extents.height(),
            );
            draw_ctx.cr_background.fill().ok();
        }

        draw_ctx.cr_text.move_to(x, y);
        draw_ctx.cr_text.show_text(text).ok();
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
        self.background = option;

        let draw_ctx = self.draw_ctx.borrow();
        draw_ctx.cr_background.set_source_rgb(
            (r as f64) / 255.,
            (g as f64) / 255.,
            (b as f64) / 255.,
        )
    }

    fn use_brush(&mut self, option: crate::ir::BrushType, r: u16, g: u16, b: u16) {
        self.brush_type = option;

        let draw_ctx = self.draw_ctx.borrow();

        draw_ctx
            .cr_brush
            .set_source(match option {
                ir::BrushType::Solid => {
                    let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, 1, 1).unwrap();
                    let cr = cairo::Context::new(&surface).unwrap();
                    cr.set_source_rgb((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
                    cr.paint().ok();
                    let pattern = cairo::SurfacePattern::create(surface);
                    pattern.set_extend(cairo::Extend::Repeat);
                    pattern
                }
                ir::BrushType::DiagonalUp => todo!(),
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
                    let pattern = cairo::SurfacePattern::create(surface);
                    pattern.set_extend(cairo::Extend::Repeat);
                    pattern
                }
            })
            .ok();
    }

    fn use_caption(&mut self, text: &str) {
        self.window.set_title(text);
    }

    fn use_coordinates(&mut self, option: crate::ir::Coordinates) {
        self.scale = match option {
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
        let draw_ctx = self.draw_ctx.borrow();

        self.underline = underline;

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

        draw_ctx.cr_text.set_font_face(&font_face);
        draw_ctx
            .cr_text
            .set_source_rgb((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);

        let extents = draw_ctx.cr_text.font_extents().unwrap();
        let mut matrix = cairo::Matrix::identity();
        if width != 0 {
            matrix.set_xx(self.scaled(width) / extents.max_x_advance());
        }
        if height != 0 {
            matrix.set_yy(self.scaled(height) / extents.height());
        }
        draw_ctx.cr_text.set_font_matrix(matrix);
    }

    fn use_pen(&mut self, option: crate::ir::PenType, width: u16, r: u16, g: u16, b: u16) {
        let draw_ctx = self.draw_ctx.borrow();

        self.pen_type = option;

        draw_ctx.cr_pen.set_dash(
            match option {
                ir::PenType::Solid => &[],
                ir::PenType::Null => &[0.],
                ir::PenType::Dash => &[24., 8.],
                ir::PenType::Dot => &[4.],
                ir::PenType::DashDot => &[12., 6., 3., 6.],
                ir::PenType::DashDotDot => &[12., 3., 3., 3., 3., 3.],
            },
            0.,
        );
        draw_ctx.cr_pen.set_line_width(width.into());
        draw_ctx.cr_background.set_line_width(width.into());
        draw_ctx
            .cr_pen
            .set_source_rgb((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
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
