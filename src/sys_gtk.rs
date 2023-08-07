use std::cell::Ref;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::f64::consts::TAU;
use std::process;
use std::rc::Rc;
use std::time;

use gtk::cairo;
use gtk::gdk::prelude::*;
use gtk::gdk_pixbuf;
use gtk::prelude::*;

use crate::ir;
use crate::vm;
use crate::vm::VMSys;

macro_rules! cairo_context_getter_and_invalidator {
    ($var: ident, $member: ident, $var_inval:ident, $cr: expr) => {
        fn $var(&self) -> Ref<cairo::Context> {
            {
                let borrowed = self.$member.borrow();
                if let Some(_) = *borrowed {
                    return Ref::map(borrowed, |cr| cr.as_ref().unwrap());
                }
            }
            {
                let mut borrowed = self.$member.borrow_mut();
                let cr = cairo::Context::new(&self.surface).unwrap();
                cr.set_antialias(cairo::Antialias::None);
                *borrowed = Some(cr);
            }
            let borrowed = self.$member.borrow();
            $cr(self, borrowed.as_ref().unwrap());
            Ref::map(borrowed, |cr| cr.as_ref().unwrap())
        }

        fn $var_inval(&self) {
            let mut borrowed = self.$member.borrow_mut();
            *borrowed = None;
        }
    };
}

macro_rules! scale_vars {
    ($draw_ctx:expr, ($($x:ident),*)) => {
        $(
            let $x = $draw_ctx.scaled($x);
        )*
    };
}

mod cairo_util {
    use super::*;

    pub fn new_surface_rgb(
        width: i32,
        height: i32,
        r: f64,
        g: f64,
        b: f64,
    ) -> (cairo::ImageSurface, cairo::Context) {
        let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width, height).unwrap();
        let cr = cairo::Context::new(&surface).unwrap();
        cr.set_source_rgb(r, g, b);
        cr.paint().ok();
        (surface, cr)
    }

    pub fn draw_pattern_diagonal_up(cr: &cairo::Context) {
        cr.move_to(0.5, 8.);
        cr.line_to(8., 0.5);
    }

    pub fn draw_pattern_diagonal_down(cr: &cairo::Context) {
        cr.move_to(0., 0.5);
        cr.line_to(7.5, 8.);
    }

    pub fn draw_pattern_horizontal(cr: &cairo::Context) {
        cr.move_to(0., 0.);
        cr.line_to(8., 0.);
    }

    pub fn draw_pattern_vertical(cr: &cairo::Context) {
        cr.move_to(0., 0.);
        cr.line_to(0., 8.);
    }
}

struct DrawCtx {
    surface: cairo::ImageSurface,
    cr_text_: RefCell<Option<cairo::Context>>,
    cr_pen_: RefCell<Option<cairo::Context>>,
    cr_background_: RefCell<Option<cairo::Context>>,
    cr_brush_: RefCell<Option<cairo::Context>>,

    text_face: cairo::FontFace,
    text_matrix: cairo::Matrix,
    text_underline: crate::ir::FontUnderline,
    text_rgb: (f64, f64, f64),

    pen_type: ir::PenType,
    pen_width: f64,
    pen_rgb: (f64, f64, f64),

    background_transparency: ir::BackgroundTransparency,
    background_rgb: (f64, f64, f64),

    brush_type: ir::BrushType,
    brush_rgb: (f64, f64, f64),

    scale: f64,
}

impl DrawCtx {
    fn new() -> Self {
        DrawCtx {
            surface: cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0).unwrap(),
            cr_text_: RefCell::new(None),
            cr_pen_: RefCell::new(None),
            cr_background_: RefCell::new(None),
            cr_brush_: RefCell::new(None),

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

            brush_type: ir::BrushType::Null,
            brush_rgb: (0., 0., 0.),

            scale: 1.,
        }
    }

    cairo_context_getter_and_invalidator!(
        cr_text,
        cr_text_,
        cr_text_inval,
        |draw_ctx: &DrawCtx, cr: &cairo::Context| {
            let (r, g, b) = draw_ctx.text_rgb;
            cr.set_source_rgb(r, g, b);
            cr.set_font_matrix(draw_ctx.text_matrix);
        }
    );

    cairo_context_getter_and_invalidator!(
        cr_pen,
        cr_pen_,
        cr_pen_inval,
        |draw_ctx: &DrawCtx, cr: &cairo::Context| {
            let (r, g, b) = draw_ctx.pen_rgb;
            cr.set_dash(
                match draw_ctx.pen_type {
                    ir::PenType::Solid => &[],
                    ir::PenType::Null => &[0.],
                    ir::PenType::Dash => &[24., 8.],
                    ir::PenType::Dot => &[4.],
                    ir::PenType::DashDot => &[12., 6., 3., 6.],
                    ir::PenType::DashDotDot => &[12., 3., 3., 3., 3., 3.],
                },
                0.,
            );
            cr.set_line_width(draw_ctx.pen_width);
            cr.set_source_rgb(r, g, b);
        }
    );

    cairo_context_getter_and_invalidator!(
        cr_background,
        cr_background_,
        cr_background_inval,
        |draw_ctx: &DrawCtx, cr: &cairo::Context| {
            let (r, g, b) = draw_ctx.background_rgb;
            cr.set_line_width(draw_ctx.pen_width);
            cr.set_source_rgb(r, g, b);
        }
    );

    cairo_context_getter_and_invalidator!(
        cr_brush,
        cr_brush_,
        cr_brush_inval,
        |draw_ctx: &DrawCtx, cr: &cairo::Context| {
            let (r, g, b) = draw_ctx.brush_rgb;
            let (bkg_r, bkg_g, bkg_b) = draw_ctx.background_rgb;
            let pattern = cairo::SurfacePattern::create(match draw_ctx.brush_type {
                ir::BrushType::Solid => cairo_util::new_surface_rgb(1, 1, r, g, b).0,
                ir::BrushType::DiagonalUp => {
                    let (surface, cr) = cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b);
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_up(&cr);
                    cr.rectangle(0., 0., 0.5, 0.5);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::DiagonalDown => {
                    let (surface, cr) = cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b);
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_down(&cr);
                    cr.rectangle(8., 0., 0.5, 0.5);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::DiagonalCross => {
                    let (surface, cr) = cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b);
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_up(&cr);
                    cairo_util::draw_pattern_diagonal_down(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Horizontal => {
                    let (surface, cr) = cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b);
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_horizontal(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Vertical => {
                    let (surface, cr) = cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b);
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_vertical(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Cross => {
                    let (surface, cr) = cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b);
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_horizontal(&cr);
                    cairo_util::draw_pattern_vertical(&cr);
                    cr.stroke().ok();
                    surface
                }
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
        }
    );

    fn resize(&mut self, width: i32, height: i32) {
        self.surface = {
            let (surface, cr) = cairo_util::new_surface_rgb(
                width,
                height,
                self.background_rgb.0,
                self.background_rgb.1,
                self.background_rgb.2,
            );
            cr.set_source_surface(&self.surface, 0., 0.).ok();
            cr.paint().ok();
            surface
        };
        *self.cr_text_.borrow_mut() = None;
        *self.cr_pen_.borrow_mut() = None;
        *self.cr_background_.borrow_mut() = None;
        *self.cr_brush_.borrow_mut() = None;
    }

    fn scaled(&self, x: u16) -> f64 {
        (x as f64) * self.scale
    }

    fn line_exec(&self, brush: bool, op: impl Fn(Ref<cairo::Context>)) {
        if brush {
            match self.brush_type {
                ir::BrushType::Solid
                | ir::BrushType::DiagonalUp
                | ir::BrushType::DiagonalDown
                | ir::BrushType::DiagonalCross
                | ir::BrushType::Horizontal
                | ir::BrushType::Vertical
                | ir::BrushType::Cross => {
                    op(self.cr_brush());
                }
                ir::BrushType::Null => {}
            }
        }
        match self.background_transparency {
            ir::BackgroundTransparency::Opaque => {
                op(self.cr_background());
            }
            ir::BackgroundTransparency::Transparent => {}
        }
        match self.pen_type {
            ir::PenType::Solid
            | ir::PenType::Dash
            | ir::PenType::Dot
            | ir::PenType::DashDot
            | ir::PenType::DashDotDot => {
                op(self.cr_pen());
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
        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;

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
        self.cr_brush().fill().ok();
        self.cr_background().stroke().ok();
        self.cr_pen().stroke().ok();
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
        drawing_area.connect_draw(move |_, cr| {
            let draw_ctx = draw_ctx_clone.borrow();
            cr.set_source_surface(draw_ctx.surface.as_ref(), 0., 0.)
                .ok();
            cr.paint().ok();
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

impl<'a> vm::VMSys<'a> for VMSysGtk {
    fn beep(&mut self) {
        self.window.window().unwrap().beep();
    }

    fn draw_arc(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16, x4: u16, y4: u16) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2, x3, y3, x4, y4));

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx);
        let theta2 = (y4 - cy).atan2(x4 - cx);

        draw_ctx.arc_path(x1, y1, x2, y2, theta1, theta2, true, false);
        draw_ctx.line_exec(false, |ctx| {
            ctx.stroke().ok();
        });
    }

    fn draw_background(&mut self) {
        let draw_ctx = self.draw_ctx.borrow();

        draw_ctx.cr_background().paint().ok();
    }

    fn draw_bitmap(&mut self, x: u16, y: u16, filename: &str) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x, y));

        let filename = filename_conv(filename);

        let pixbuf = gdk_pixbuf::Pixbuf::from_file(filename).unwrap();
        let surface = pixbuf
            .create_surface(1, self.window.window().as_ref())
            .unwrap();

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
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2, x3, y3, x4, y4));

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx); //TODO: scale before atan
        let theta2 = (y4 - cy).atan2(x4 - cx);

        let pts = draw_ctx.arc_path(x1, y1, x2, y2, theta1, theta2, true, true);
        draw_ctx.line_exec(true, |ctx| {
            ctx.line_to(pts.0, pts.1);
        });

        draw_ctx.draw();
    }

    fn draw_ellipse(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));

        draw_ctx.arc_path(x1, y1, x2, y2, TAU, 0.0, true, true);
        draw_ctx.draw();
    }

    fn draw_flood(&mut self, x: u16, y: u16, r: u16, g: u16, b: u16) {
        //TODO
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x, y));

        let tgt = [r as u8, g as u8, b as u8];

        let width = draw_ctx.surface.width() as usize;
        let height = draw_ctx.surface.height() as usize;

        let mut z: Option<cairo::ImageSurface> = None;

        draw_ctx
            .surface
            .with_data(|data| {
                let mut mask: Vec<u8> = (0..(data.len() / 4)).map(|_| 0u8).collect();
                let mut q: Vec<(usize, usize)> = vec![(x as usize, y as usize)];
                while let Some((x, y)) = q.pop() {
                    let i = x + y * width;
                    if mask[i] == 0 && data[(i * 4)..(i * 4 + 3)] != tgt {
                        mask[i] = 255;
                        if x > 0 {
                            q.push((x - 1, y));
                        }
                        if x < width - 1 {
                            q.push((x + 1, y));
                        }
                        if y > 0 {
                            q.push((x, y - 1));
                        }
                        if y < height - 1 {
                            q.push((x, y + 1));
                        }
                    }
                }
                z = Some(
                    cairo::ImageSurface::create_for_data(
                        mask,
                        cairo::Format::A8,
                        width as i32,
                        height as i32,
                        width as i32,
                    )
                    .unwrap(),
                );
            })
            .ok();

        draw_ctx.cr_brush().mask_surface(&z.unwrap(), 0., 0.).ok();
    }

    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));

        draw_ctx.line_exec(false, |ctx| {
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
        });
        draw_ctx.line_exec(false, |ctx| {
            ctx.stroke().ok();
        });
    }

    fn draw_number(&mut self, x: u16, y: u16, n: u16) {
        self.draw_text(x, y, n.to_string().as_str());
    }

    fn draw_pie(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16, x4: u16, y4: u16) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2, x3, y3, x4, y4));

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx);
        let theta2 = (y4 - cy).atan2(x4 - cx);

        let pts = draw_ctx.arc_path(x1, y1, x2, y2, theta1, theta2, true, true);
        draw_ctx.line_exec(true, |ctx| {
            ctx.line_to(cx, cy);
            ctx.line_to(pts.0, pts.1);
        });
        draw_ctx.draw();
    }

    fn draw_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));

        draw_ctx.line_exec(true, |ctx| {
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y1);
            ctx.line_to(x2, y2);
            ctx.line_to(x1, y2);
            ctx.line_to(x1, y1);
        });
        draw_ctx.draw();
    }

    fn draw_round_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, x3: u16, y3: u16) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2, x3, y3));

        draw_ctx.arc_path(x1, y1, x1 + x3, y1 + y3, PI * 1.5, PI, false, true);
        draw_ctx.line_exec(true, |ctx| {
            ctx.line_to(x1, y2 - y3 / 2.);
        });
        draw_ctx.arc_path(x1, y2 - y3, x1 + x3, y2, PI, PI * 0.5, false, true);
        draw_ctx.line_exec(true, |ctx| {
            ctx.line_to(x2 - x3 / 2., y2);
        });
        draw_ctx.arc_path(x2 - x3, y2 - y3, x2, y2, PI * 0.5, 0., false, true);
        draw_ctx.line_exec(true, |ctx| {
            ctx.line_to(x2, y1 + y3 / 2.);
        });
        draw_ctx.arc_path(x2 - x3, y1, x2, y1 + y3, 0., PI * -0.5, false, true);
        draw_ctx.line_exec(true, |ctx| {
            ctx.line_to(x1 + x3 / 2., y1);
        });

        draw_ctx.draw();
    }

    fn draw_sized_bitmap(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, filename: &str) {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));
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
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x, y));

        if let ir::BackgroundTransparency::Opaque = draw_ctx.background_transparency {
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

    fn set_keyboard(&mut self, params: Vec<vm::SetKeyboardParam>) {
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
        draw_ctx.background_transparency = option;
        draw_ctx.background_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_background_inval();
        draw_ctx.cr_brush_inval();
    }

    fn use_brush(&mut self, option: crate::ir::BrushType, r: u16, g: u16, b: u16) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();
        draw_ctx.brush_type = option;
        draw_ctx.brush_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_brush_inval();
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

        draw_ctx.text_underline = underline;

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
        draw_ctx.text_face = font_face;
        draw_ctx.text_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.text_matrix = matrix;
        draw_ctx.cr_text_inval();

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
        draw_ctx.text_matrix = matrix;
        draw_ctx.cr_text_inval();
    }

    fn use_pen(&mut self, option: crate::ir::PenType, width: u16, r: u16, g: u16, b: u16) {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        draw_ctx.pen_type = option;
        draw_ctx.pen_width = width.into();
        draw_ctx.pen_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_pen_inval();
        draw_ctx.cr_background_inval();
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
