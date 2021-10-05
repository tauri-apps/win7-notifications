use tao::{
  event::{Event, StartCause},
  event_loop::EventLoop,
};
use win7_notifications::{Notification, Timeout};
fn main() {
  let event_loop = EventLoop::new();
  let icon = include_bytes!("icon.ico");

  event_loop.run(move |event, _, _| match event {
    Event::NewEvents(e) if e == StartCause::Init => {
      Notification::new()
        .appname("App name")
        .summary("Critical Error")
        .body("Just kidding, this is just the notification example 1.")
        .icon(icon.to_vec())
        .timeout(Timeout::Never)
        .show()
        .unwrap();
      Notification::new()
        .appname("App name")
        .summary("Critical Error")
        .body("Just kidding, this is just the notification example 2.")
        .icon(icon.to_vec())
        .timeout(Timeout::Default)
        .show()
        .unwrap();
      Notification::new()
        .appname("App name")
        .summary("Critical Error")
        .body("Just kidding, this is just the notification example 3.")
        .icon(icon.to_vec())
        .timeout(Timeout::Never)
        .show()
        .unwrap();
      Notification::new()
        .appname("App name")
        .summary("Critical Error")
        .body("Just kidding, this is just the notification example 4.")
        .icon(icon.to_vec())
        .timeout(Timeout::Default)
        .show()
        .unwrap();
    }
    _ => (),
  });
}
