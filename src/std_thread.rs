///! Implementation of the dynamic timeout with the std thread library
use anyhow::{bail, Result};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

type DurationVec = Arc<Mutex<Vec<Duration>>>;
pub struct DynTimeout {
    thread: Option<JoinHandle<()>>,
    cancelled: Arc<AtomicBool>,
    durations: DurationVec,
}

impl DynTimeout {
    pub fn new(dur: Duration, callback: fn() -> ()) -> Self {
        let durations: DurationVec = Arc::new(Mutex::new(vec![Duration::ZERO, dur]));
        let thread_vec = durations.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let thread_cancelled = cancelled.clone();
        Self {
            thread: Some(thread::spawn(move || {
                while let Some(dur) = thread_vec.lock().unwrap().pop() {
                    thread::sleep(dur)
                }
                if thread_cancelled.load(Ordering::Relaxed) {
                    callback();
                }
            })),
            cancelled,
            durations,
        }
    }
    pub fn add(&self, dur: Duration) -> Result<()> {
        match self.durations.lock() {
            Ok(mut durations) => {
                if durations.is_empty() {
                    bail!("Timeout already reached")
                }
                durations.push(dur);
                Ok(())
            }
            Err(err) => bail!(err.to_string()),
        }
    }
    pub fn sub(&self, dur: Duration) -> Result<()> {
        let mut durations = match self.durations.lock() {
            Ok(durations) => {
                if durations.is_empty() {
                    bail!("Timeout already reached")
                } else {
                    durations
                }
            }
            Err(err) => bail!(err.to_string()),
        };
        let mut pop_dur = Duration::default();
        while pop_dur < dur && durations.len() > 1 {
            pop_dur += durations.pop().unwrap();
        }
        if pop_dur > dur {
            durations.push(pop_dur - dur);
        }
        Ok(())
    }
    pub fn cancel(&mut self) -> Result<()> {
        self.cancelled.store(true, Ordering::Relaxed);
        match self.durations.lock() {
            Ok(mut durations) => durations.clear(),
            Err(err) => bail!(err.to_string()),
        };
        self.join()
    }
    pub fn join(&mut self) -> Result<()> {
        if self.thread.is_none() {
            return Ok(());
        }
        match self.thread.take() {
            Some(thread) => match thread.join() {
                Ok(_) => Ok(()),
                Err(_) => bail!("Cannot join dyn-timeout"),
            },
            None => bail!("Cannot join dyn-timeout"),
        }
    }
}

impl Drop for DynTimeout {
    fn drop(&mut self) {
        self.join().unwrap()
    }
}
