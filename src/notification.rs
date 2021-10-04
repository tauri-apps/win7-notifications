// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use std::{
  thread::{sleep, spawn},
  time::Duration,
};

use crate::{
  timeout::Timeout,
  util::{self, Color},
};
use bindings::Windows::Win32::{
  Foundation as w32f,
  Graphics::Gdi as w32gdi,
  System::{Com, LibraryLoader},
  UI::{Controls, WindowsAndMessaging as w32wm},
};
use windows::*;

const CLOSE_BTN_ID: isize = 554;
/// notification width
const NOTI_W: i32 = 360;
/// notification height
const NOTI_H: i32 = 170;
/// notification margin
const NOTI_M: i32 = 16;
/// notification icon size (width/height)
const NOTI_ICON_S: i32 = 16;
/// notification window bg color
const WND_CLR: Color = Color(50, 57, 69);
/// used for notification summary(title)
const TITILE_CLR: Color = Color(255, 255, 255);
/// used for notification body
const SUBTITLE_CLR: Color = Color(255, 255, 255);

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Notification {
  pub icon: Vec<u8>,
  pub appname: String,
  pub summary: String,
  pub body: String,
  pub timeout: Timeout,
}

impl Default for Notification {
  fn default() -> Notification {
    Notification {
      appname: util::current_exe_name(),
      summary: String::new(),
      body: String::new(),
      icon: Vec::new(),
      timeout: Timeout::Default,
    }
  }
}

impl Notification {
  /// Constructs a new Notification.
  ///
  /// Most fields are empty by default, only `appname` is initialized with the name of the current
  /// executable.
  pub fn new() -> Notification {
    Notification::default()
  }

  /// Overwrite the appname field used for Notification.
  pub fn appname(&mut self, appname: &str) -> &mut Notification {
    self.appname = appname.to_owned();
    self
  }

  /// Set the `summary`.
  ///
  /// Often acts as title of the notification. For more elaborate content use the `body` field.
  pub fn summary(&mut self, summary: &str) -> &mut Notification {
    self.summary = summary.to_owned();
    self
  }

  /// Set the content of the `body` field.
  ///
  /// Multiline textual content of the notification.
  /// Each line should be treated as a paragraph.
  /// html markup is not supported.
  pub fn body(&mut self, body: &str) -> &mut Notification {
    self.body = body.to_owned();
    self
  }

  /// Set the `icon` field must be `.ico` byte array.
  pub fn icon(&mut self, icon: Vec<u8>) -> &mut Notification {
    self.icon = icon;
    self
  }

  /// Set the `timeout` field.
  pub fn timeout(&mut self, timeout: Timeout) -> &mut Notification {
    self.timeout = timeout;
    self
  }

  /// Shows the Notification.
  ///
  /// Requires an win32 event_loop to be running, otherwise the notification will close immediately.
  pub fn show(&self) -> Result<()> {
    unsafe {
      let hinstance = LibraryLoader::GetModuleHandleW(w32f::PWSTR::default());

      let mut class_name = String::from("win7-notify-rust");
      let wnd_class = w32wm::WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: w32f::PWSTR(class_name.as_mut_ptr() as _),
        hInstance: hinstance,
        hbrBackground: w32gdi::CreateSolidBrush(WND_CLR.to_int()),
        ..Default::default()
      };
      w32wm::RegisterClassW(&wnd_class);

      let monitor_info = util::get_monitor_info(util::primary_monitor());
      let w32f::RECT { right, bottom, .. } = monitor_info.__AnonymousBase_winuser_L13558_C43.rcWork;

      let data = WindowData {
        window: w32f::HWND::default(),
        close_btn: w32f::HWND::default(),
        appname_control: w32f::HWND::default(),
        summary_control: w32f::HWND::default(),
        body_control: w32f::HWND::default(),
        mouse_hovering_close_btn: false,
        notification: self.clone(),
      };

      let hwnd = w32wm::CreateWindowExW(
        w32wm::WS_EX_TOPMOST,
        w32f::PWSTR(class_name.as_mut_ptr() as _),
        "win7-notify-rust-window",
        w32wm::WS_SYSMENU | w32wm::WS_CAPTION | w32wm::WS_VISIBLE,
        right - NOTI_W - 15,
        bottom - NOTI_H - 15,
        NOTI_W,
        NOTI_H,
        w32f::HWND::default(),
        w32wm::HMENU::default(),
        hinstance,
        Box::into_raw(Box::new(data)) as _,
      );

      if hwnd.is_invalid() {
        //
      }

      // skip notificcation window from taskbar
      let _ = Com::CoInitialize(std::ptr::null());
      util::skip_taskbar(hwnd);

      w32wm::ShowWindow(hwnd, w32wm::SW_SHOWDEFAULT);

      let timeout = self.timeout;
      spawn(move || {
        sleep(Duration::from_millis(timeout.into()));
        if timeout != Timeout::Never {
          w32wm::ShowWindow(hwnd, w32wm::SW_HIDE);
          w32wm::CloseWindow(hwnd);
        };
      });
    }

    Ok(())
  }
}

struct WindowData {
  window: w32f::HWND,
  close_btn: w32f::HWND,
  appname_control: w32f::HWND,
  body_control: w32f::HWND,
  summary_control: w32f::HWND,
  mouse_hovering_close_btn: bool,
  notification: Notification,
}

pub unsafe extern "system" fn window_proc(
  hwnd: w32f::HWND,
  msg: u32,
  wparam: w32f::WPARAM,
  lparam: w32f::LPARAM,
) -> w32f::LRESULT {
  let mut userdata = util::get_window_long_ptr(hwnd, w32wm::GWL_USERDATA);

  match msg {
    w32wm::WM_NCCREATE => {
      if userdata == 0 {
        let createstruct = &*(lparam.0 as *const w32wm::CREATESTRUCTW);
        userdata = createstruct.lpCreateParams as isize;
        util::set_window_long_ptr(hwnd, w32wm::GWL_USERDATA, userdata);
      }
      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    // makes the window borderless
    w32wm::WM_NCCALCSIZE => w32f::LRESULT(0),

    w32wm::WM_CREATE => {
      let userdata = userdata as *mut WindowData;

      // fill userdata with missing info
      (*userdata).window = hwnd;

      // create the notification close button
      (*userdata).close_btn = w32wm::CreateWindowExW(
        w32wm::WINDOW_EX_STYLE(0),
        "BUTTON",
        "",
        w32wm::WS_VISIBLE
          | w32wm::WS_CHILD
          | w32wm::WINDOW_STYLE((w32wm::BS_OWNERDRAW | w32wm::BS_CENTER) as _),
        NOTI_W - NOTI_ICON_S - NOTI_M,
        NOTI_M,
        NOTI_ICON_S,
        NOTI_ICON_S,
        hwnd,
        w32wm::HMENU(CLOSE_BTN_ID),
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );

      // create the notification appname text control
      (*userdata).appname_control = w32wm::CreateWindowExW(
        w32wm::WINDOW_EX_STYLE(0),
        "STATIC",
        "",
        w32wm::WS_VISIBLE | w32wm::WS_CHILD | w32wm::WINDOW_STYLE(w32wm::SS_OWNERDRAW as _),
        NOTI_M + NOTI_ICON_S + (NOTI_M / 4),
        NOTI_M,
        NOTI_W - NOTI_M + NOTI_ICON_S + (NOTI_M / 4) - NOTI_ICON_S - NOTI_M,
        NOTI_ICON_S,
        hwnd,
        w32wm::HMENU::default(),
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );

      // create the notification summary(title) text control
      (*userdata).summary_control = w32wm::CreateWindowExW(
        w32wm::WINDOW_EX_STYLE(0),
        "STATIC",
        "",
        w32wm::WS_VISIBLE | w32wm::WS_CHILD | w32wm::WINDOW_STYLE(w32wm::SS_OWNERDRAW as _),
        NOTI_M,
        NOTI_M + NOTI_ICON_S + (NOTI_M / 2),
        NOTI_W - NOTI_M - (NOTI_M / 2) - NOTI_ICON_S - NOTI_M,
        NOTI_ICON_S,
        hwnd,
        w32wm::HMENU::default(),
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );

      // create the notification body text control
      (*userdata).body_control = w32wm::CreateWindowExW(
        w32wm::WINDOW_EX_STYLE(0),
        "STATIC",
        "",
        w32wm::WS_VISIBLE | w32wm::WS_CHILD | w32wm::WINDOW_STYLE(w32wm::SS_OWNERDRAW as _),
        NOTI_M,
        NOTI_M + NOTI_ICON_S + (NOTI_M / 2) + NOTI_ICON_S + (NOTI_M / 2),
        NOTI_W - NOTI_M - (NOTI_M / 2) - NOTI_ICON_S - NOTI_M,
        NOTI_H - NOTI_M - NOTI_ICON_S - (NOTI_M / 2) - NOTI_ICON_S - (NOTI_M / 2),
        hwnd,
        w32wm::HMENU::default(),
        w32f::HINSTANCE::default(),
        std::ptr::null_mut(),
      );
      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_CTLCOLORBTN => {
      let userdata = userdata as *mut WindowData;

      if lparam.0 == (*userdata).close_btn.0 {
        // change the close button control background color to match the window color
        return w32f::LRESULT(w32gdi::CreateSolidBrush(WND_CLR.to_int()).0 as _);
      }

      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_DRAWITEM => {
      let userdata = userdata as *mut WindowData;
      let lpds = lparam.0 as *mut Controls::DRAWITEMSTRUCT;

      w32gdi::SetBkMode((*lpds).hDC, w32gdi::TRANSPARENT);

      // draw notification close button
      if (*lpds).hwndItem == (*userdata).close_btn {
        w32gdi::SetTextColor((*lpds).hDC, Color(150, 150, 150).to_int());
        w32gdi::TextOutW((*lpds).hDC, 5, 1, "x", 1);
      }

      // draw notification app name
      if (*lpds).hwndItem == (*userdata).appname_control {
        util::set_font((*lpds).hDC, "Segeo UI", 15, 400);
        w32gdi::SetTextColor((*lpds).hDC, TITILE_CLR.to_int());
        w32gdi::TextOutW(
          (*lpds).hDC,
          5,
          1,
          (*userdata).notification.appname.clone(),
          (*userdata).notification.appname.len() as _,
        );
      }

      // draw notification summary (title)
      if (*lpds).hwndItem == (*userdata).summary_control {
        util::set_font((*lpds).hDC, "Segeo UI", 18, 700);
        w32gdi::SetTextColor((*lpds).hDC, TITILE_CLR.to_int());
        w32gdi::TextOutW(
          (*lpds).hDC,
          0,
          0,
          (*userdata).notification.summary.clone(),
          (*userdata).notification.summary.len() as _,
        );
      }

      // draw notification body
      if (*lpds).hwndItem == (*userdata).body_control {
        util::set_font((*lpds).hDC, "Segeo UI", 18, 400);
        w32gdi::SetTextColor((*lpds).hDC, SUBTITLE_CLR.to_int());
        let mut rc = w32f::RECT::default();
        w32wm::GetClientRect((*lpds).hwndItem, &mut rc);
        w32gdi::DrawTextW(
          (*lpds).hDC,
          (*userdata).notification.body.clone(),
          (*userdata).notification.body.len() as _,
          &mut rc,
          w32gdi::DT_LEFT | w32gdi::DT_EXTERNALLEADING | w32gdi::DT_WORDBREAK,
        );
      }

      w32f::LRESULT(true.into())
    }

    w32wm::WM_COMMAND => {
      if util::get_loword(wparam.0 as _) == CLOSE_BTN_ID as u16 {
        w32wm::ShowWindow(hwnd, w32wm::SW_HIDE);
        w32wm::CloseWindow(hwnd);
      }
      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_PAINT => {
      let userdata = userdata as *mut WindowData;

      // draw notification icon
      if let Some(hicon) = util::get_hicon_from_buffer(&(*userdata).notification.icon, 16, 16) {
        let mut ps = w32gdi::PAINTSTRUCT::default();
        let hdc = w32gdi::BeginPaint(hwnd, &mut ps);
        w32wm::DrawIconEx(
          hdc,
          NOTI_M,
          NOTI_M,
          hicon,
          NOTI_ICON_S,
          NOTI_ICON_S,
          0,
          w32gdi::HBRUSH::default(),
          w32wm::DI_NORMAL,
        );
        w32gdi::EndPaint(hwnd, &mut ps);
      }

      w32wm::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    _ => w32wm::DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}
