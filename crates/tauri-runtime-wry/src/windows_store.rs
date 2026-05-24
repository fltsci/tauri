// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::WindowWrapper;
use std::{
  collections::BTreeMap,
  fmt,
  sync::{RwLock, TryLockError},
};
use tauri_runtime::window::WindowId;

type WindowMap = BTreeMap<WindowId, WindowWrapper>;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
  /// The store's lock could not be acquired (already held in a conflicting mode).
  Borrow(LockKind),
  /// The store's lock was poisoned by a thread panicking while holding it.
  Poisoned(LockKind),
  WindowNotFound(WindowId),
}

#[derive(Debug, Clone, Copy)]
pub enum LockKind {
  Read,
  Write,
}

impl fmt::Display for LockKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      LockKind::Read => "read",
      LockKind::Write => "write",
    })
  }
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Borrow(kind) => write!(f, "windows_store {kind} lock unavailable"),
      Error::Poisoned(kind) => write!(f, "windows_store {kind} lock poisoned"),
      Error::WindowNotFound(id) => write!(f, "Window not in store: {id:?}"),
    }
  }
}

impl From<Error> for tauri_runtime::Error {
  fn from(value: Error) -> Self {
    Self::WindowsStore(Box::new(value))
  }
}

fn map_try_lock<G>(kind: LockKind, result: std::result::Result<G, TryLockError<G>>) -> Result<G> {
  result.map_err(|e| match e {
    TryLockError::Poisoned(_) => Error::Poisoned(kind),
    TryLockError::WouldBlock => Error::Borrow(kind),
  })
}

#[cfg(feature = "tracing-borrows")]
#[track_caller]
fn trace_borrow(kind: LockKind) {
  let caller = std::panic::Location::caller();
  tracing::trace!(
    target: "tauri::runtime::wry::windows_store",
    kind = %kind,
    line = caller.line(),
    column = caller.column(),
    thread = format!("{:?}", std::thread::current().id()),
  );
}

#[cfg(not(feature = "tracing-borrows"))]
#[inline(always)]
fn trace_borrow(_kind: LockKind) {}

#[derive(Default)]
pub struct WindowsStore(RwLock<WindowMap>);

impl fmt::Debug for WindowsStore {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowsStore(opaque)").finish()
  }
}

impl WindowsStore {
  #[track_caller]
  pub fn window<F, T>(&self, id: WindowId, f: F) -> Result<T>
  where
    F: FnOnce(&WindowWrapper) -> T,
  {
    trace_borrow(LockKind::Read);
    let store = map_try_lock(LockKind::Read, self.0.try_read())?;
    let window = store.get(&id).ok_or(Error::WindowNotFound(id))?;
    Ok(f(window))
  }

  #[track_caller]
  pub fn window_mut<F, T>(&self, id: WindowId, f: F) -> Result<T>
  where
    F: FnOnce(&mut WindowWrapper) -> T,
  {
    trace_borrow(LockKind::Write);
    let mut store = map_try_lock(LockKind::Write, self.0.try_write())?;
    let window = store.get_mut(&id).ok_or(Error::WindowNotFound(id))?;
    Ok(f(window))
  }

  #[track_caller]
  pub fn store<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&WindowMap) -> T,
  {
    trace_borrow(LockKind::Read);
    let store = map_try_lock(LockKind::Read, self.0.try_read())?;
    Ok(f(&store))
  }

  #[track_caller]
  pub fn store_mut<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&mut WindowMap) -> T,
  {
    trace_borrow(LockKind::Write);
    let mut store = map_try_lock(LockKind::Write, self.0.try_write())?;
    Ok(f(&mut store))
  }

  pub fn insert(&self, id: WindowId, window: WindowWrapper) -> Result<Option<WindowWrapper>> {
    self.store_mut(|s| s.insert(id, window))
  }

  /// Removes the window from the store and return `true` if the store is now empty.
  pub fn remove(&self, id: WindowId) -> Result<bool> {
    self.store_mut(|s| {
      s.remove(&id);
      s.is_empty()
    })
  }
}

