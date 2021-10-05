// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Send Windows 10 styled notifications on Windows 7.
//!
//! # Examples
//!
//! # Example 1: Simple Notification
//!
//! ```no_run
//! # use win7_notifications::*;
//! Notification::new()
//!     .appname("App name")
//!     .summary("Critical Error")
//!     .body("Just kidding, this is just the notification example.")
//!     .icon(icon.to_vec())
//!     .timeout(Timeout::Default) // 5000 milliseconds
//!     .show().unwrap();
//! ```
//!
//! # Example 2: Presistent Notification
//!
//! ```no_run
//! # use win7_notifications::*;
//! Notification::new()
//!     .appname("App name")
//!     .summary("Critical Error")
//!     .body("Just kidding, this is just the notification example.")
//!     .icon(icon.to_vec())
//!     .timeout(Timeout::Never)
//!     .show().unwrap();
//! ```
//!

windows::include_bindings!();

mod notification;
mod timeout;
mod util;

pub use crate::{notification::Notification, timeout::Timeout};
