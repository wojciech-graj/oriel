use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::f64::consts::TAU;
use std::process;
use std::rc::Rc;
use std::time;

use gtk::cairo;
use gtk::gdk;
use gtk::gdk::prelude::*;
use gtk::gdk_pixbuf;
use gtk::prelude::*;
use thiserror::Error;

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
    ) -> Result<(cairo::ImageSurface, cairo::Context), cairo::Error> {
        let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width, height)?;
        let cr = cairo::Context::new(&surface)?;
        cr.set_source_rgb(r, g, b);
        cr.paint()?;
        Ok((surface, cr))
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
    fn new() -> Result<Self, cairo::Error> {
        Ok(DrawCtx {
            surface: cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)?,
            cr_text_: RefCell::new(None),
            cr_pen_: RefCell::new(None),
            cr_background_: RefCell::new(None),
            cr_brush_: RefCell::new(None),

            text_face: cairo::FontFace::toy_create(
                "Sans",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal,
            )?,
            text_matrix: {
                let mut matrix = cairo::Matrix::identity(); //TODO: get system default?
                matrix.set_xx(10.);
                matrix.set_yy(10.);
                matrix
            },
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
        })
    }

    cairo_context_getter_and_invalidator!(
        cr_text,
        cr_text_,
        cr_text_inval,
        |draw_ctx: &DrawCtx, cr: &cairo::Context| {
            let (r, g, b) = draw_ctx.text_rgb;
            cr.set_font_face(&draw_ctx.text_face);
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
                ir::BrushType::Solid => cairo_util::new_surface_rgb(1, 1, r, g, b).unwrap().0,
                ir::BrushType::DiagonalUp => {
                    let (surface, cr) =
                        cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_up(&cr);
                    cr.rectangle(0., 0., 0.5, 0.5);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::DiagonalDown => {
                    let (surface, cr) =
                        cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_down(&cr);
                    cr.rectangle(8., 0., 0.5, 0.5);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::DiagonalCross => {
                    let (surface, cr) =
                        cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_up(&cr);
                    cairo_util::draw_pattern_diagonal_down(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Horizontal => {
                    let (surface, cr) =
                        cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_horizontal(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Vertical => {
                    let (surface, cr) =
                        cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_vertical(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Cross => {
                    let (surface, cr) =
                        cairo_util::new_surface_rgb(8, 8, bkg_r, bkg_g, bkg_b).unwrap();
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

    fn resize(&mut self, width: i32, height: i32) -> Result<(), cairo::Error> {
        self.surface = {
            let (surface, cr) = cairo_util::new_surface_rgb(
                width,
                height,
                self.background_rgb.0,
                self.background_rgb.1,
                self.background_rgb.2,
            )?;
            cr.set_source_surface(&self.surface, 0., 0.)?;
            cr.paint()?;
            surface
        };
        *self.cr_text_.borrow_mut() = None;
        *self.cr_pen_.borrow_mut() = None;
        *self.cr_background_.borrow_mut() = None;
        *self.cr_brush_.borrow_mut() = None;
        Ok(())
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

    fn draw(&self) -> Result<(), cairo::Error> {
        self.cr_brush().fill()?;
        self.cr_background().stroke()?;
        self.cr_pen().stroke()?;
        Ok(())
    }

    fn stroke(&self) -> Result<(), cairo::Error> {
        self.cr_background().stroke()?;
        self.cr_pen().stroke()?;
        Ok(())
    }
}

struct MouseRegion<'a> {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    callbacks: &'a ir::MouseCallbacks<'a>,
}

impl<'a> MouseRegion<'a> {
    fn contains(&self, x: f64, y: f64) -> bool {
        self.x1 <= x && self.y1 < y && self.x2 >= x && self.y2 >= y
    }
}

struct InputQueue {
    keyboard: Vec<vm::Key>,
    mouse: Vec<(f64, f64)>,
    menu: Vec<usize>,
    closed: bool,
}

impl InputQueue {
    fn new() -> Self {
        InputQueue {
            keyboard: Vec::new(),
            mouse: Vec::new(),
            menu: Vec::new(),
            closed: false,
        }
    }

    fn clear(&mut self) {
        self.keyboard = Vec::new();
        self.mouse = Vec::new();
        self.menu = Vec::new();
    }
}

struct InputCtx<'a> {
    keyboard: HashMap<vm::Key, ir::Identifier<'a>>,
    mouse: Vec<MouseRegion<'a>>,
    menu: HashMap<usize, ir::Identifier<'a>>,
    queue: Rc<RefCell<InputQueue>>,
}

impl<'a> InputCtx<'a> {
    fn new() -> Self {
        InputCtx {
            keyboard: HashMap::new(),
            mouse: Vec::new(),
            menu: HashMap::new(),
            queue: Rc::new(RefCell::new(InputQueue::new())),
        }
    }

    fn clear_queue(&self) {
        self.queue.borrow_mut().clear();
    }

    fn process_queue(&self, scale: f64) -> Option<vm::Input<'a>> {
        {
            let queue = self.queue.borrow();
            if queue.closed {
                return Some(vm::Input::End);
            }
            for key in queue.keyboard.iter() {
                if let Some(&label) = self.keyboard.get(key) {
                    return Some(vm::Input::Goto(label));
                }
            }
            for mouse in queue.mouse.iter() {
                for region in self.mouse.iter() {
                    if region.contains(mouse.0, mouse.1) {
                        return Some(vm::Input::Mouse {
                            callbacks: region.callbacks,
                            x: (mouse.0 / scale) as u16,
                            y: (mouse.1 / scale) as u16,
                        });
                    }
                }
            }
            for menu in queue.menu.iter() {
                if let Some(&label) = self.menu.get(menu) {
                    return Some(vm::Input::Goto(label));
                }
            }
        }
        self.clear_queue();
        None
    }
}

pub struct VMSysGtk<'a> {
    window: gtk::Window,
    help: gtk::MenuItem,
    menu_bar: gtk::MenuBar,
    draw_ctx: Rc<RefCell<DrawCtx>>,
    input_ctx: InputCtx<'a>,
    wait_mode: ir::WaitMode,
}

impl<'a> VMSysGtk<'a> {
    pub fn new(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        gtk::init()?;

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_default_size(640, 480);
        window.set_resizable(false);
        window.set_title(format!("Oriel - {}", filename).as_str());
        let logo = gdk::gdk_pixbuf::Pixbuf::from_file("res/LOGO.png")?;
        window.set_icon(Some(&logo));

        let mainbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
        window.add(&mainbox);

        let about = {
            let about = gtk::AboutDialog::new();
            about.set_logo(Some(&logo));
            about.set_icon(Some(&logo));
            about.connect_delete_event(|about, _| {
                about.hide();
                Inhibit(true)
            });
            about
        };

        let help = {
            let help = gtk::MenuItem::with_mnemonic("_Help");
            help.set_right_justified(true);
            help.connect_activate(move |_| {
                about.show_all();
            });
            help
        };

        let menu_bar = {
            let menu_bar = gtk::MenuBar::new();
            menu_bar.append(&help);
            menu_bar
        };
        mainbox.pack_start(&menu_bar, false, true, 0);

        let drawing_area = gtk::DrawingArea::new();
        mainbox.pack_start(&drawing_area, true, true, 0);

        let draw_ctx = Rc::new(RefCell::new(DrawCtx::new()?));

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
                .resize(rect.width(), rect.height())
                .ok();
        });

        let input_ctx = InputCtx::new();

        let queue_clone = input_ctx.queue.clone();
        window.connect_key_press_event(move |_, event_key| {
            let mut queue = queue_clone.borrow_mut();
            queue.keyboard.extend(eventkey_conv(event_key));
            Inhibit(false)
        });

        let queue_clone = input_ctx.queue.clone();
        drawing_area.connect_button_press_event(move |_, event_button| {
            if let Some(coords) = event_button.coords() {
                let mut queue = queue_clone.borrow_mut();
                queue.mouse.push(coords);
            }
            Inhibit(false)
        });

        let queue_clone = input_ctx.queue.clone();
        window.connect_delete_event(move |_, _| {
            queue_clone.borrow_mut().closed = true;
            Inhibit(false)
        });

        drawing_area.add_events(gdk::EventMask::BUTTON_PRESS_MASK);

        window.show_all();
        window.set_mnemonics_visible(true);

        let mut sys = VMSysGtk {
            window,
            menu_bar,
            help,
            draw_ctx,
            input_ctx,
            wait_mode: ir::WaitMode::Null,
        };

        sys.use_coordinates(ir::Coordinates::Metric)?;

        Ok(sys)
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

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
enum Error {
    #[error("Failed to get window")]
    WindowMissingError,
    #[error("Failed to create cairo surface")]
    SurfaceCreateError,
    #[error("Failed to get monitor")]
    MonitorMissingError,
}

impl<'a> vm::VMSys<'a> for VMSysGtk<'a> {
    fn beep(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.window
            .window()
            .ok_or_else(|| Error::WindowMissingError)?
            .beep();
        Ok(())
    }

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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2, x3, y3, x4, y4));

        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = (y3 - cy).atan2(x3 - cx);
        let theta2 = (y4 - cy).atan2(x4 - cx);

        draw_ctx.arc_path(x1, y1, x2, y2, theta1, theta2, true, false);
        draw_ctx.stroke()?;
        Ok(())
    }

    fn draw_background(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        draw_ctx.cr_background().paint()?;
        Ok(())
    }

    fn draw_bitmap(
        &mut self,
        x: u16,
        y: u16,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x, y));

        let filename = filename_conv(filename);

        let pixbuf = gdk_pixbuf::Pixbuf::from_file(filename)?;
        let surface = pixbuf
            .create_surface(1, self.window.window().as_ref())
            .ok_or_else(|| Error::SurfaceCreateError)?;

        let cr = cairo::Context::new(draw_ctx.surface.as_ref())?;
        cr.set_source_surface(&surface, x, y)?;
        cr.paint()?;
        Ok(())
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
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        draw_ctx.draw()?;
        Ok(())
    }

    fn draw_ellipse(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));

        draw_ctx.arc_path(x1, y1, x2, y2, TAU, 0.0, true, true);
        draw_ctx.draw()?;
        Ok(())
    }

    fn draw_flood(
        &mut self,
        x: u16,
        y: u16,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x, y));

        let tgt = [r as u8, g as u8, b as u8];

        let width = draw_ctx.surface.width() as usize;
        let height = draw_ctx.surface.height() as usize;

        let mut mask_surface: Option<Result<cairo::ImageSurface, cairo::Error>> = None;

        draw_ctx.surface.with_data(|data| {
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
            mask_surface = Some(cairo::ImageSurface::create_for_data(
                mask,
                cairo::Format::A8,
                width as i32,
                height as i32,
                width as i32,
            ));
        })?;

        let mask_surface = mask_surface.unwrap()?;

        draw_ctx.cr_brush().mask_surface(&mask_surface, 0., 0.)?;
        Ok(())
    }

    fn draw_line(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));

        draw_ctx.line_exec(false, |ctx| {
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
        });
        draw_ctx.stroke()?;
        Ok(())
    }

    fn draw_number(&mut self, x: u16, y: u16, n: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_text(x, y, n.to_string().as_str())
    }

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
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        draw_ctx.draw()?;
        Ok(())
    }

    fn draw_rectangle(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));

        draw_ctx.line_exec(true, |ctx| {
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y1);
            ctx.line_to(x2, y2);
            ctx.line_to(x1, y2);
            ctx.line_to(x1, y1);
        });
        draw_ctx.draw()?;
        Ok(())
    }

    fn draw_round_rectangle(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        x3: u16,
        y3: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        draw_ctx.draw()?;
        Ok(())
    }

    fn draw_sized_bitmap(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x1, y1, x2, y2));
        let filename = filename_conv(filename);

        let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_size(
            filename,
            (x1 - x2).abs() as i32,
            (y1 - y2).abs() as i32,
        )?;
        let surface = pixbuf
            .create_surface(1, self.window.window().as_ref())
            .ok_or_else(|| Error::SurfaceCreateError)?;

        let cr = cairo::Context::new(draw_ctx.surface.as_ref())?;
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
        cr.set_source_surface(&surface, x1.min(x2), y1.min(y2))?;
        cr.paint()?;
        Ok(())
    }

    fn draw_text(&mut self, x: u16, y: u16, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();

        scale_vars!(draw_ctx, (x, y));

        if let ir::BackgroundTransparency::Opaque = draw_ctx.background_transparency {
            let text_extents = draw_ctx.cr_text().text_extents(text)?;
            let font_extents = draw_ctx.cr_text().font_extents()?;
            draw_ctx.cr_background().rectangle(
                x,
                y - font_extents.ascent(),
                text_extents.width(),
                font_extents.height(),
            );
            draw_ctx.cr_background().fill()?;
        }

        draw_ctx.cr_text().move_to(x, y);
        draw_ctx.cr_text().show_text(text)?;
        Ok(())
    }

    fn message_box(
        &mut self,
        typ: crate::ir::MessageBoxType,
        default_button: u16, //TODO
        icon: crate::ir::MessageBoxIcon,
        text: &str,
        caption: &str,
    ) -> Result<u16, Box<dyn std::error::Error>> {
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

        Ok(if let gtk::ResponseType::Other(x) = response {
            x
        } else {
            default_button
        })
    }

    fn run(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let command = command_conv(command);

        process::Command::new("sh").arg("-c").arg(command).spawn()?;
        Ok(())
    }

    fn set_keyboard(
        &mut self,
        params: HashMap<vm::Key, ir::Identifier<'a>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.input_ctx.keyboard = params;
        Ok(())
    }

    fn set_menu(
        &mut self,
        menu: &Vec<ir::MenuCategory<'a>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.menu_bar
            .children()
            .iter()
            .for_each(|child| self.menu_bar.remove(child));
        self.input_ctx.menu = HashMap::new();
        for category in menu.iter() {
            self.menu_bar.append(&{
                let item = menu_item_conv(&category.item, &mut self.input_ctx);
                if category.members.len() > 0 {
                    item.set_submenu(Some(&{
                        let submenu = gtk::Menu::new();
                        category.members.iter().for_each(|member| {
                            match member {
                                ir::MenuMember::Item(subitem) => {
                                    submenu.append(&menu_item_conv(&subitem, &mut self.input_ctx))
                                }
                                ir::MenuMember::Separator => {
                                    submenu.append(&gtk::SeparatorMenuItem::new())
                                }
                            };
                        });
                        submenu
                    }));
                }
                item
            });
        }
        self.menu_bar.append(&self.help);
        self.window.show_all();
        self.window.set_mnemonics_visible(true);
        Ok(())
    }

    fn set_mouse(
        &mut self,
        regions: Vec<vm::MouseRegion<'a>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_ctx = self.draw_ctx.borrow();
        self.input_ctx.mouse = regions
            .iter()
            .map(|region| MouseRegion {
                x1: draw_ctx.scaled(region.x1),
                y1: draw_ctx.scaled(region.y1),
                x2: draw_ctx.scaled(region.x2),
                y2: draw_ctx.scaled(region.y2),
                callbacks: region.callbacks,
            })
            .collect();
        Ok(())
    }

    fn set_wait_mode(
        &mut self,
        mode: crate::ir::WaitMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.wait_mode = mode;
        Ok(())
    }

    fn set_window(
        &mut self,
        option: crate::ir::SetWindowOption,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match option {
            ir::SetWindowOption::Maximize => self.window.maximize(),
            ir::SetWindowOption::Minimize => self.window.iconify(),
            ir::SetWindowOption::Restore => {
                self.window.unmaximize();
                self.window.deiconify();
            }
        }
        Ok(())
    }

    fn use_background(
        &mut self,
        option: crate::ir::BackgroundTransparency,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut draw_ctx = self.draw_ctx.borrow_mut();
        draw_ctx.background_transparency = option;
        draw_ctx.background_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_background_inval();
        draw_ctx.cr_brush_inval();
        Ok(())
    }

    fn use_brush(
        &mut self,
        option: crate::ir::BrushType,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut draw_ctx = self.draw_ctx.borrow_mut();
        draw_ctx.brush_type = option;
        draw_ctx.brush_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_brush_inval();
        Ok(())
    }

    fn use_caption(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.window.set_title(text);
        Ok(())
    }

    fn use_coordinates(
        &mut self,
        option: crate::ir::Coordinates,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        draw_ctx.scale = match option {
            ir::Coordinates::Pixel => 1.,
            ir::Coordinates::Metric => {
                let window_gdk = self
                    .window
                    .window()
                    .ok_or_else(|| Error::WindowMissingError)?;
                let monitor = window_gdk
                    .display()
                    .monitor_at_window(&window_gdk)
                    .ok_or_else(|| Error::MonitorMissingError)?;
                (monitor.geometry().width() as f64) / (monitor.width_mm() as f64)
            }
        };
        Ok(())
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
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        )?;

        let mut matrix = cairo::Matrix::identity();
        draw_ctx.text_face = font_face;
        draw_ctx.text_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.text_matrix = matrix;
        draw_ctx.cr_text_inval();

        if width == 0 && height == 0 {
            return Ok(());
        }

        let extents = draw_ctx.cr_text().font_extents()?;
        if width != 0 {
            matrix.set_xx(draw_ctx.scaled(width) / extents.max_x_advance());
            println!("{}", draw_ctx.scaled(width) / extents.max_x_advance());
        }
        if height != 0 {
            matrix.set_yy(draw_ctx.scaled(height) / extents.height());
        }
        draw_ctx.text_matrix = matrix;
        draw_ctx.cr_text_inval();
        Ok(())
    }

    fn use_pen(
        &mut self,
        option: crate::ir::PenType,
        width: u16,
        r: u16,
        g: u16,
        b: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut draw_ctx = self.draw_ctx.borrow_mut();

        draw_ctx.pen_type = option;
        draw_ctx.pen_width = width.into();
        draw_ctx.pen_rgb = ((r as f64) / 255., (g as f64) / 255., (b as f64) / 255.);
        draw_ctx.cr_pen_inval();
        draw_ctx.cr_background_inval();
        Ok(())
    }

    fn wait_input(
        &mut self,
        milliseconds: Option<u16>,
    ) -> Result<Option<vm::Input<'a>>, Box<dyn std::error::Error>> {
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
                        if self.input_ctx.queue.borrow().closed {
                            return Ok(Some(vm::Input::End));
                        }
                    }
                } else {
                    while gtk::events_pending() {
                        gtk::main_iteration();
                    }
                    self.input_ctx.clear_queue();
                    let scale = self.draw_ctx.borrow().scale;
                    while self.window.is_visible() {
                        while gtk::events_pending() {
                            gtk::main_iteration();
                        }
                        if let Some(input) = self.input_ctx.process_queue(scale) {
                            return Ok(Some(input));
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
                        if self.input_ctx.queue.borrow().closed {
                            return Ok(Some(vm::Input::End));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

fn eventkey_conv(event: &gdk::EventKey) -> Vec<vm::Key> {
    let keys = match event.keyval() {
        gdk::keys::constants::BackSpace => Some((ir::VirtualKey::BackSpace, None)),
        gdk::keys::constants::Tab => Some((ir::VirtualKey::Tab, None)),
        gdk::keys::constants::Return => Some((ir::VirtualKey::Enter, None)),
        gdk::keys::constants::Shift_L | gdk::keys::constants::Shift_R => {
            Some((ir::VirtualKey::Shift, None))
        }
        gdk::keys::constants::Control_L | gdk::keys::constants::Control_R => {
            Some((ir::VirtualKey::Ctrl, None))
        }
        gdk::keys::constants::Alt_L | gdk::keys::constants::Alt_R => {
            Some((ir::VirtualKey::Alt, None))
        }
        gdk::keys::constants::Pause => Some((ir::VirtualKey::Pause, None)),
        gdk::keys::constants::Caps_Lock => Some((ir::VirtualKey::CapsLock, None)),
        gdk::keys::constants::Escape => Some((ir::VirtualKey::Escape, None)),
        gdk::keys::constants::space => Some((ir::VirtualKey::Space, Some(' '))),
        gdk::keys::constants::Page_Up => Some((ir::VirtualKey::PgUp, None)),
        gdk::keys::constants::Page_Down => Some((ir::VirtualKey::PgDn, None)),
        gdk::keys::constants::End => Some((ir::VirtualKey::End, None)),
        gdk::keys::constants::Home => Some((ir::VirtualKey::Home, None)),
        gdk::keys::constants::Left => Some((ir::VirtualKey::LeftArrow, None)),
        gdk::keys::constants::Up => Some((ir::VirtualKey::UpArrow, None)),
        gdk::keys::constants::Right => Some((ir::VirtualKey::RightArrow, None)),
        gdk::keys::constants::Down => Some((ir::VirtualKey::DownArrow, None)),
        gdk::keys::constants::_3270_PrintScreen => Some((ir::VirtualKey::PrintScreen, None)),
        gdk::keys::constants::Insert => Some((ir::VirtualKey::Insert, None)),
        gdk::keys::constants::Delete => Some((ir::VirtualKey::Delete, None)),
        gdk::keys::constants::Arabic_0 => Some((ir::VirtualKey::AlNum('0'), Some('0'))),
        gdk::keys::constants::parenright => Some((ir::VirtualKey::AlNum('0'), Some(')'))),
        gdk::keys::constants::Arabic_1 => Some((ir::VirtualKey::AlNum('1'), Some('1'))),
        gdk::keys::constants::exclam => Some((ir::VirtualKey::AlNum('0'), Some('!'))),
        gdk::keys::constants::Arabic_2 => Some((ir::VirtualKey::AlNum('2'), Some('2'))),
        gdk::keys::constants::at => Some((ir::VirtualKey::AlNum('0'), Some('@'))),
        gdk::keys::constants::Arabic_3 => Some((ir::VirtualKey::AlNum('3'), Some('3'))),
        gdk::keys::constants::numbersign => Some((ir::VirtualKey::AlNum('0'), Some('#'))),
        gdk::keys::constants::Arabic_4 => Some((ir::VirtualKey::AlNum('4'), Some('4'))),
        gdk::keys::constants::dollar => Some((ir::VirtualKey::AlNum('0'), Some('$'))),
        gdk::keys::constants::Arabic_5 => Some((ir::VirtualKey::AlNum('5'), Some('5'))),
        gdk::keys::constants::percent => Some((ir::VirtualKey::AlNum('0'), Some('%'))),
        gdk::keys::constants::Arabic_6 => Some((ir::VirtualKey::AlNum('6'), Some('6'))),
        gdk::keys::constants::asciicircum => Some((ir::VirtualKey::AlNum('0'), Some('^'))),
        gdk::keys::constants::Arabic_7 => Some((ir::VirtualKey::AlNum('7'), Some('7'))),
        gdk::keys::constants::ampersand => Some((ir::VirtualKey::AlNum('0'), Some('&'))),
        gdk::keys::constants::Arabic_8 => Some((ir::VirtualKey::AlNum('8'), Some('8'))),
        gdk::keys::constants::asterisk => Some((ir::VirtualKey::AlNum('0'), Some('*'))),
        gdk::keys::constants::Arabic_9 => Some((ir::VirtualKey::AlNum('9'), Some('9'))),
        gdk::keys::constants::parenleft => Some((ir::VirtualKey::AlNum('0'), Some('('))),
        gdk::keys::constants::A => Some((ir::VirtualKey::AlNum('A'), Some('A'))),
        gdk::keys::constants::B => Some((ir::VirtualKey::AlNum('B'), Some('B'))),
        gdk::keys::constants::C => Some((ir::VirtualKey::AlNum('C'), Some('C'))),
        gdk::keys::constants::D => Some((ir::VirtualKey::AlNum('D'), Some('D'))),
        gdk::keys::constants::E => Some((ir::VirtualKey::AlNum('E'), Some('E'))),
        gdk::keys::constants::F => Some((ir::VirtualKey::AlNum('F'), Some('F'))),
        gdk::keys::constants::G => Some((ir::VirtualKey::AlNum('G'), Some('G'))),
        gdk::keys::constants::H => Some((ir::VirtualKey::AlNum('H'), Some('H'))),
        gdk::keys::constants::I => Some((ir::VirtualKey::AlNum('I'), Some('I'))),
        gdk::keys::constants::J => Some((ir::VirtualKey::AlNum('J'), Some('J'))),
        gdk::keys::constants::K => Some((ir::VirtualKey::AlNum('K'), Some('K'))),
        gdk::keys::constants::L => Some((ir::VirtualKey::AlNum('L'), Some('L'))),
        gdk::keys::constants::M => Some((ir::VirtualKey::AlNum('M'), Some('M'))),
        gdk::keys::constants::N => Some((ir::VirtualKey::AlNum('N'), Some('N'))),
        gdk::keys::constants::O => Some((ir::VirtualKey::AlNum('O'), Some('O'))),
        gdk::keys::constants::P => Some((ir::VirtualKey::AlNum('P'), Some('P'))),
        gdk::keys::constants::Q => Some((ir::VirtualKey::AlNum('Q'), Some('Q'))),
        gdk::keys::constants::R => Some((ir::VirtualKey::AlNum('R'), Some('R'))),
        gdk::keys::constants::S => Some((ir::VirtualKey::AlNum('S'), Some('S'))),
        gdk::keys::constants::T => Some((ir::VirtualKey::AlNum('T'), Some('T'))),
        gdk::keys::constants::U => Some((ir::VirtualKey::AlNum('U'), Some('U'))),
        gdk::keys::constants::V => Some((ir::VirtualKey::AlNum('V'), Some('V'))),
        gdk::keys::constants::W => Some((ir::VirtualKey::AlNum('W'), Some('W'))),
        gdk::keys::constants::X => Some((ir::VirtualKey::AlNum('X'), Some('X'))),
        gdk::keys::constants::Y => Some((ir::VirtualKey::AlNum('Y'), Some('Y'))),
        gdk::keys::constants::Z => Some((ir::VirtualKey::AlNum('Z'), Some('Z'))),
        gdk::keys::constants::a => Some((ir::VirtualKey::AlNum('A'), Some('a'))),
        gdk::keys::constants::b => Some((ir::VirtualKey::AlNum('B'), Some('b'))),
        gdk::keys::constants::c => Some((ir::VirtualKey::AlNum('C'), Some('c'))),
        gdk::keys::constants::d => Some((ir::VirtualKey::AlNum('D'), Some('d'))),
        gdk::keys::constants::e => Some((ir::VirtualKey::AlNum('E'), Some('e'))),
        gdk::keys::constants::f => Some((ir::VirtualKey::AlNum('F'), Some('f'))),
        gdk::keys::constants::g => Some((ir::VirtualKey::AlNum('G'), Some('g'))),
        gdk::keys::constants::h => Some((ir::VirtualKey::AlNum('H'), Some('h'))),
        gdk::keys::constants::i => Some((ir::VirtualKey::AlNum('I'), Some('i'))),
        gdk::keys::constants::j => Some((ir::VirtualKey::AlNum('J'), Some('j'))),
        gdk::keys::constants::k => Some((ir::VirtualKey::AlNum('K'), Some('k'))),
        gdk::keys::constants::l => Some((ir::VirtualKey::AlNum('L'), Some('l'))),
        gdk::keys::constants::m => Some((ir::VirtualKey::AlNum('M'), Some('m'))),
        gdk::keys::constants::n => Some((ir::VirtualKey::AlNum('N'), Some('n'))),
        gdk::keys::constants::o => Some((ir::VirtualKey::AlNum('O'), Some('o'))),
        gdk::keys::constants::p => Some((ir::VirtualKey::AlNum('P'), Some('p'))),
        gdk::keys::constants::q => Some((ir::VirtualKey::AlNum('Q'), Some('q'))),
        gdk::keys::constants::r => Some((ir::VirtualKey::AlNum('R'), Some('r'))),
        gdk::keys::constants::s => Some((ir::VirtualKey::AlNum('S'), Some('s'))),
        gdk::keys::constants::t => Some((ir::VirtualKey::AlNum('T'), Some('t'))),
        gdk::keys::constants::u => Some((ir::VirtualKey::AlNum('U'), Some('u'))),
        gdk::keys::constants::v => Some((ir::VirtualKey::AlNum('V'), Some('v'))),
        gdk::keys::constants::w => Some((ir::VirtualKey::AlNum('W'), Some('w'))),
        gdk::keys::constants::x => Some((ir::VirtualKey::AlNum('X'), Some('x'))),
        gdk::keys::constants::y => Some((ir::VirtualKey::AlNum('Y'), Some('y'))),
        gdk::keys::constants::z => Some((ir::VirtualKey::AlNum('Z'), Some('z'))),
        gdk::keys::constants::KP_0 => Some((ir::VirtualKey::NumPad('0'), Some('0'))),
        gdk::keys::constants::KP_1 => Some((ir::VirtualKey::NumPad('1'), Some('1'))),
        gdk::keys::constants::KP_2 => Some((ir::VirtualKey::NumPad('2'), Some('2'))),
        gdk::keys::constants::KP_3 => Some((ir::VirtualKey::NumPad('3'), Some('3'))),
        gdk::keys::constants::KP_4 => Some((ir::VirtualKey::NumPad('4'), Some('4'))),
        gdk::keys::constants::KP_5 => Some((ir::VirtualKey::NumPad('5'), Some('5'))),
        gdk::keys::constants::KP_6 => Some((ir::VirtualKey::NumPad('6'), Some('6'))),
        gdk::keys::constants::KP_7 => Some((ir::VirtualKey::NumPad('7'), Some('7'))),
        gdk::keys::constants::KP_8 => Some((ir::VirtualKey::NumPad('8'), Some('8'))),
        gdk::keys::constants::KP_9 => Some((ir::VirtualKey::NumPad('9'), Some('9'))),
        gdk::keys::constants::KP_Multiply => Some((ir::VirtualKey::NumPad('*'), Some('*'))),
        gdk::keys::constants::KP_Add => Some((ir::VirtualKey::NumPad('+'), Some('+'))),
        gdk::keys::constants::KP_Subtract => Some((ir::VirtualKey::NumPad('-'), Some('-'))),
        gdk::keys::constants::KP_Decimal => Some((ir::VirtualKey::NumPad('.'), Some('.'))),
        gdk::keys::constants::KP_Divide => Some((ir::VirtualKey::NumPad('/'), Some('/'))),
        gdk::keys::constants::F1 => Some((ir::VirtualKey::F(1), None)),
        gdk::keys::constants::F2 => Some((ir::VirtualKey::F(2), None)),
        gdk::keys::constants::F3 => Some((ir::VirtualKey::F(3), None)),
        gdk::keys::constants::F4 => Some((ir::VirtualKey::F(4), None)),
        gdk::keys::constants::F5 => Some((ir::VirtualKey::F(5), None)),
        gdk::keys::constants::F6 => Some((ir::VirtualKey::F(6), None)),
        gdk::keys::constants::F7 => Some((ir::VirtualKey::F(7), None)),
        gdk::keys::constants::F8 => Some((ir::VirtualKey::F(8), None)),
        gdk::keys::constants::F9 => Some((ir::VirtualKey::F(9), None)),
        gdk::keys::constants::F10 => Some((ir::VirtualKey::F(10), None)),
        gdk::keys::constants::F11 => Some((ir::VirtualKey::F(11), None)),
        gdk::keys::constants::F12 => Some((ir::VirtualKey::F(12), None)),
        gdk::keys::constants::F13 => Some((ir::VirtualKey::F(13), None)),
        gdk::keys::constants::F14 => Some((ir::VirtualKey::F(14), None)),
        gdk::keys::constants::F15 => Some((ir::VirtualKey::F(15), None)),
        gdk::keys::constants::F16 => Some((ir::VirtualKey::F(16), None)),
        gdk::keys::constants::Num_Lock => Some((ir::VirtualKey::NumLock, None)),
        gdk::keys::constants::Scroll_Lock => Some((ir::VirtualKey::ScrollLock, None)),
        gdk::keys::constants::colon => Some((ir::VirtualKey::ColonOrSemiColon, Some(':'))),
        gdk::keys::constants::semicolon => Some((ir::VirtualKey::ColonOrSemiColon, Some(';'))),
        gdk::keys::constants::plus => Some((ir::VirtualKey::PlusOrEqual, Some('+'))),
        gdk::keys::constants::equal => Some((ir::VirtualKey::PlusOrEqual, Some('='))),
        gdk::keys::constants::less => Some((ir::VirtualKey::LessOrComma, Some('<'))),
        gdk::keys::constants::comma => Some((ir::VirtualKey::LessOrComma, Some(','))),
        gdk::keys::constants::underscore => Some((ir::VirtualKey::UnderscoreOrHyphen, Some('_'))),
        gdk::keys::constants::hyphen => Some((ir::VirtualKey::UnderscoreOrHyphen, Some('-'))),
        gdk::keys::constants::greater => Some((ir::VirtualKey::GreaterOrPeriod, Some('>'))),
        gdk::keys::constants::period => Some((ir::VirtualKey::GreaterOrPeriod, Some('.'))),
        gdk::keys::constants::question => Some((ir::VirtualKey::QuestionOrSlash, Some('?'))),
        gdk::keys::constants::slash => Some((ir::VirtualKey::QuestionOrSlash, Some('/'))),
        gdk::keys::constants::asciitilde => {
            Some((ir::VirtualKey::TildeOrBackwardsSingleQuote, Some('~')))
        }
        gdk::keys::constants::grave => {
            Some((ir::VirtualKey::TildeOrBackwardsSingleQuote, Some('`')))
        }
        gdk::keys::constants::bracketleft => {
            Some((ir::VirtualKey::LeftCurlyOrLeftSquare, Some('[')))
        }
        gdk::keys::constants::braceleft => Some((ir::VirtualKey::LeftCurlyOrLeftSquare, Some('{'))),
        gdk::keys::constants::bar => Some((ir::VirtualKey::PipeOrBackslash, Some('|'))),
        gdk::keys::constants::backslash => Some((ir::VirtualKey::PipeOrBackslash, Some('\\'))),
        gdk::keys::constants::bracketright => {
            Some((ir::VirtualKey::RightCurlyOrRightSquare, Some(']')))
        }
        gdk::keys::constants::braceright => {
            Some((ir::VirtualKey::RightCurlyOrRightSquare, Some('}')))
        }
        gdk::keys::constants::quotedbl => {
            Some((ir::VirtualKey::DoubleQuoteOrSingleQuote, Some('"')))
        }
        gdk::keys::constants::apostrophe => {
            Some((ir::VirtualKey::DoubleQuoteOrSingleQuote, Some('\'')))
        }
        _ => None,
    };

    match keys {
        Some((virt, physical)) => match physical {
            Some(physical) => vec![
                vm::Key::Virtual(virt),
                vm::Key::Physical(ir::PhysicalKey {
                    chr: physical,
                    ctrl: event.state().contains(gdk::ModifierType::CONTROL_MASK),
                }),
            ],
            None => vec![vm::Key::Virtual(virt)],
        },
        None => Vec::new(),
    }
}

fn menu_item_conv<'a>(item: &ir::MenuItem<'a>, input_ctx: &mut InputCtx<'a>) -> gtk::MenuItem {
    let menu_item = if item.name.contains("&") {
        gtk::MenuItem::with_mnemonic(&item.name.replace("&", "_"))
    } else {
        gtk::MenuItem::with_label(item.name)
    };
    if let Some(label) = item.label {
        let queue_clone = input_ctx.queue.clone();
        let key = input_ctx.menu.len();
        menu_item.connect_activate(move |_| queue_clone.borrow_mut().menu.push(key));
        input_ctx.menu.insert(key, label);
    }
    menu_item
}
