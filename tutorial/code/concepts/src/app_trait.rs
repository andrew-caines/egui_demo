//! Examples for the `app-trait` concept page.
//!
//! The gallery's own `App::ui` calls each example's `show(ui)` once per frame, so an example
//! here sees exactly the frames a real app would see.

use std::time::Duration;

use eframe::egui;

use crate::{Demo, Example};

pub fn examples() -> Vec<Example> {
    vec![
        Example {
            concept: "app-trait",
            name: "When ui runs",
            when: "Use this when you want to see how often egui calls your code, and what makes it call again.",
            make: || Box::new(FrameCounter::default()),
        },
        Example {
            concept: "app-trait",
            name: "Ticking clock",
            when: "Use this when something on screen changes with time, but not every frame.",
            make: || Box::new(TickingClock::default()),
        },
    ]
}

/// Counts the frames it has been drawn in. egui runs a frame on input (a mouse move, a key) or
/// on request, so with the checkbox off the count only rises while you move the mouse.
#[derive(Default)]
pub struct FrameCounter {
    frames: u64,
    repaint_every_frame: bool,
}

impl FrameCounter {
    /// How many times `show` has run.
    pub fn frames(&self) -> u64 {
        self.frames
    }
}

impl Demo for FrameCounter {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.frames += 1;
        ui.label(format!("This code has run {} times.", self.frames));
        ui.checkbox(&mut self.repaint_every_frame, "Repaint every frame");
        if self.repaint_every_frame {
            // Ask for another frame as soon as this one is done: an animation loop.
            ui.ctx().request_repaint();
        }
    }
}

/// Shows how long it has been running, in whole seconds. It asks for one frame when the next
/// second starts, so it stays correct while costing one frame a second.
#[derive(Default)]
pub struct TickingClock {
    /// egui's time when the clock was first drawn.
    started: Option<f64>,
    elapsed: f64,
}

impl TickingClock {
    /// The whole seconds shown last frame.
    pub fn whole_seconds(&self) -> u64 {
        self.elapsed.floor() as u64
    }
}

impl Demo for TickingClock {
    fn show(&mut self, ui: &mut egui::Ui) {
        // Seconds since the app started, set by eframe at the start of every frame.
        let now = ui.input(|input| input.time);
        let started = *self.started.get_or_insert(now);
        self.elapsed = now - started;
        ui.label(format!("Running for {} s", self.whole_seconds()));

        // Wake up when the number changes, and not before.
        let until_next_second = 1.0 - self.elapsed.fract();
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64(until_next_second));
    }
}
