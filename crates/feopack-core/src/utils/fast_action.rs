use std::{mem, thread};

pub fn fast_drop<T>(src: T)
where
  T: Send + 'static,
{
  thread::spawn(move || {
    mem::drop(src);
  });
}

pub fn fast_set<T>(dest: &mut T, src: T)
where
  T: Send + 'static,
{
  let old = mem::replace(dest, src);
  fast_drop(old);
}
