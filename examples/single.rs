// Copyright 2020-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

use win7_notifications::{Notification, Timeout};
use winit::{
    event::{Event, StartCause},
    event_loop::{ControlFlow, EventLoop},
};
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let (icon, w, h) = load_icon(Path::new(path));

    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Poll);

            match event {
                Event::NewEvents(StartCause::Init) => {
                    Notification::new()
                        .appname("App name")
                        .summary("Critical Error")
                        .body("Just kidding, this is just the notification example.")
                        .icon(icon.clone(), w, h)
                        .timeout(Timeout::Default)
                        .show()
                        .unwrap();
                }
                _ => (),
            }
        })
        .unwrap();
}

fn load_icon(path: &Path) -> (Vec<u8>, u32, u32) {
    let image = image::open(path)
        .expect("Failed to open icon path")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    (rgba, width, height)
}
