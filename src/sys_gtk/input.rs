use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ir, vm};

pub struct MouseRegion<'a> {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub callbacks: &'a ir::MouseCallbacks<'a>,
}

impl<'a> MouseRegion<'a> {
    fn contains(&self, x: f64, y: f64) -> bool {
        self.x1 <= x && self.y1 < y && self.x2 >= x && self.y2 >= y
    }
}

#[derive(Default)]
pub struct InputQueue {
    pub keyboard: Vec<vm::Key>,
    pub mouse: Vec<(f64, f64)>,
    pub menu: Vec<usize>,
    pub closed: bool,
}

impl InputQueue {
    fn clear(&mut self) {
        self.keyboard = Vec::new();
        self.mouse = Vec::new();
        self.menu = Vec::new();
    }
}

#[derive(Default)]
pub struct InputCtx<'a> {
    pub keyboard: HashMap<vm::Key, ir::Identifier<'a>>,
    pub mouse: Vec<MouseRegion<'a>>,
    pub menu: HashMap<usize, ir::Identifier<'a>>,
    pub queue: Rc<RefCell<InputQueue>>,
}

impl<'a> InputCtx<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_queue(&self) {
        self.queue.borrow_mut().clear();
    }

    pub fn process_queue(&self, scale: f64) -> Option<vm::Input<'a>> {
        {
            let queue = self.queue.borrow();
            if queue.closed {
                return Some(vm::Input::End);
            }
            for key in &queue.keyboard {
                if let Some(&label) = self.keyboard.get(key) {
                    return Some(vm::Input::Goto(label));
                }
            }
            for mouse in &queue.mouse {
                for region in &self.mouse {
                    if region.contains(mouse.0, mouse.1) {
                        return Some(vm::Input::Mouse {
                            callbacks: region.callbacks,
                            x: (mouse.0 / scale) as u16,
                            y: (mouse.1 / scale) as u16,
                        });
                    }
                }
            }
            for menu in &queue.menu {
                if let Some(&label) = self.menu.get(menu) {
                    return Some(vm::Input::Goto(label));
                }
            }
        }
        self.clear_queue();
        None
    }
}
