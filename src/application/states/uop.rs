

use crate::application::wm::{State, StaticState, Signal};
use crate::Ssd1351;
use crate::system::system::System;

use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;

use mwatch_kernel_api::InputEvent;


pub struct UopState {
    buffer: String<U256>,
}

impl Default for UopState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl State for UopState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        display.draw(
               Image16BPP::new(include_bytes!("../../../data/uop.raw"), 48, 64)
                   .translate(Coord::new(32, 32))
                   .into_iter(),
         );
        None
    }

    fn input(&mut self, _system: &mut System, _display: &mut Ssd1351, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Left => Some(Signal::Previous),
            InputEvent::Right => Some(Signal::Next),
            _ => None
        }
    }
}

impl StaticState for UopState {}
