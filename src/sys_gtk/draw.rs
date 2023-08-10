use std::cell::Ref;
use std::cell::RefCell;
use std::f64::consts::TAU;

use gtk::cairo;

use crate::ir;

mod cairo_util {
    use gtk::cairo;

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

macro_rules! cairo_context_getter_and_invalidator {
    ($var: ident, $member: ident, $var_inval:ident, $cr: expr) => {
        pub fn $var(&self) -> Ref<cairo::Context> {
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

        pub fn $var_inval(&self) {
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

pub struct DrawCtx {
    pub surface: cairo::ImageSurface,
    cr_text_: RefCell<Option<cairo::Context>>,
    cr_pen_: RefCell<Option<cairo::Context>>,
    cr_background_: RefCell<Option<cairo::Context>>,
    cr_brush_: RefCell<Option<cairo::Context>>,

    pub text_face: cairo::FontFace,
    pub text_height_mul: Option<f64>,
    pub text_width: Option<f64>,
    pub text_underline: crate::ir::FontUnderline,
    pub text_rgb: (f64, f64, f64),

    pub pen_type: ir::PenType,
    pub pen_width: f64,
    pub pen_rgb: (f64, f64, f64),

    pub background_transparency: ir::BackgroundTransparency,
    pub background_rgb: (f64, f64, f64),

    pub brush_type: ir::BrushType,
    pub brush_rgb: (f64, f64, f64),

    pub scale: f64,
}

impl DrawCtx {
    pub fn new() -> Result<Self, cairo::Error> {
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
            text_height_mul: None,
            text_width: None,
            text_underline: ir::FontUnderline::NoUnderline,
            text_rgb: (0., 0., 0.),

            pen_type: ir::PenType::Solid,
            pen_width: 1.,
            pen_rgb: (0., 0., 0.),

            background_transparency: ir::BackgroundTransparency::Opaque,
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
            if let Some(height_mul) = draw_ctx.text_height_mul {
                let mut mat = cairo::Matrix::identity();
                mat.set_yy(height_mul);
                cr.set_font_matrix(mat);
            } else {
                cr.set_font_size(18.);
            }
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
                    ir::PenType::Null => &[0., 1.],
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
            let pattern = cairo::SurfacePattern::create(match draw_ctx.brush_type {
                ir::BrushType::Solid => cairo_util::new_surface_rgb(1, 1, r, g, b).unwrap().0,
                ir::BrushType::DiagonalUp => {
                    let (surface, cr) = draw_ctx.create_pattern_surface(8, 8).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_up(&cr);
                    cr.rectangle(0., 0., 0.5, 0.5);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::DiagonalDown => {
                    let (surface, cr) = draw_ctx.create_pattern_surface(8, 8).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_down(&cr);
                    cr.rectangle(8., 0., 0.5, 0.5);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::DiagonalCross => {
                    let (surface, cr) = draw_ctx.create_pattern_surface(8, 8).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_diagonal_up(&cr);
                    cairo_util::draw_pattern_diagonal_down(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Horizontal => {
                    let (surface, cr) = draw_ctx.create_pattern_surface(8, 8).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_horizontal(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Vertical => {
                    let (surface, cr) = draw_ctx.create_pattern_surface(8, 8).unwrap();
                    cr.set_antialias(cairo::Antialias::None);
                    cr.set_source_rgb(r, g, b);
                    cairo_util::draw_pattern_vertical(&cr);
                    cr.stroke().ok();
                    surface
                }
                ir::BrushType::Cross => {
                    let (surface, cr) = draw_ctx.create_pattern_surface(8, 8).unwrap();
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

    fn create_pattern_surface(
        &self,
        width: i32,
        height: i32,
    ) -> Result<(cairo::ImageSurface, cairo::Context), cairo::Error> {
        if let ir::BackgroundTransparency::Opaque = self.background_transparency {
            let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width, height)?;
            let cr = cairo::Context::new(&surface)?;
            let (r, g, b) = self.background_rgb;
            cr.set_source_rgb(r, g, b);
            cr.paint().ok();
            Ok((surface, cr))
        } else {
            let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)?;
            let cr = cairo::Context::new(&surface)?;
            Ok((surface, cr))
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), cairo::Error> {
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

    pub fn scaled(&self, x: u16) -> f64 {
        f64::from(x) * self.scale
    }

    pub fn line_exec(&self, brush: bool, op: impl Fn(Ref<cairo::Context>)) {
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

    pub fn arc_path(
        &self,
        cx: f64,
        cy: f64,
        sclx: f64,
        scly: f64,
        theta1: f64,
        theta2: f64,
        mv: bool,
        brush: bool,
    ) -> (f64, f64) {
        const DTHETA: f64 = -0.1;

        let startx = cx + sclx * theta1.cos();
        let starty = cy + scly * theta1.sin();
        let endx = cx + sclx * theta2.cos();
        let endy = cy + scly * theta2.sin();

        if mv {
            self.line_exec(brush, |ctx| {
                ctx.move_to(startx, starty);
            });
        }
        let mut theta = if theta1 > theta2 {
            theta1
        } else {
            theta1 + TAU
        };
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

    pub fn arc_path_rect_bound(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        x4: f64,
        y4: f64,
        brush: bool,
    ) -> (f64, f64) {
        let sclx = (x2 - x1) / 2.;
        let scly = (y2 - y1) / 2.;
        let cx = (x2 + x1) / 2.;
        let cy = (y2 + y1) / 2.;
        let theta1 = ((y3 - cy) / scly).atan2((x3 - cx) / sclx);
        let theta2 = ((y4 - cy) / scly).atan2((x4 - cx) / sclx);
        self.arc_path(cx, cy, sclx, scly, theta1, theta2, true, brush)
    }

    pub fn draw(&self) -> Result<(), cairo::Error> {
        self.cr_brush().fill()?;
        self.cr_background().stroke()?;
        self.cr_pen().stroke()?;
        Ok(())
    }

    pub fn stroke(&self) -> Result<(), cairo::Error> {
        self.cr_background().stroke()?;
        self.cr_pen().stroke()?;
        Ok(())
    }
}
