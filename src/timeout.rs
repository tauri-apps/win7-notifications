// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Describes the timeout of a notification
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Timeout {
  /// Expires according to server default.
  ///
  /// Whatever that might be...
  Default,

  /// Do not expire, user will have to close this manually.
  Never,

  /// Expire after n milliseconds.
  Milliseconds(u32),
}
impl From<Timeout> for u64 {
  fn from(timeout: Timeout) -> Self {
    match timeout {
      Timeout::Default => 5000,
      Timeout::Never => 0,
      Timeout::Milliseconds(ms) => ms as _,
    }
  }
}

impl Default for Timeout {
  fn default() -> Self {
    Timeout::Default
  }
}
