///! Implementation of the dynamic timeout with the std thread library
use anyhow::{bail, Result};
use tokio::sync::Mutex;
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::Duration,
};

type DurationVec = Arc<Mutex<Vec<Duration>>>;
pub struct DynTimeout {
    cancelled: Arc<AtomicBool>,
    durations: DurationVec,
}

impl DynTimeout {
    pub async fn new(dur: Duration, callback: fn() -> ()) -> Self {
        let durations: DurationVec = Arc::new(Mutex::new(vec![Duration::ZERO, dur]));
        let thread_vec = durations.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let thread_cancelled = cancelled.clone();
        tokio::task::spawn(async move {
            while let Some(dur) = thread_vec.lock().await.pop() {
                tokio::time::sleep(dur).await
            }
            if thread_cancelled.load(Ordering::Relaxed) {
                callback();
            }
        });
        Self {
            cancelled,
            durations,
        }
    }
    pub async fn add(&self, dur: Duration) -> Result<()> {
        let mut durations = self.durations.lock().await;
        if durations.is_empty() {
            bail!("Timeout already reached")
        }
        durations.push(dur);
        Ok(())
    }
    pub async fn sub(&self, dur: Duration) -> Result<()> {
        let mut durations = self.durations.lock().await;
        if durations.is_empty() {
            bail!("Timeout already reached")
        }
        let mut pop_dur = Duration::default();
        while pop_dur < dur && durations.len() > 1 {
            pop_dur += durations.pop().unwrap();
        }
        if pop_dur > dur {
            durations.push(pop_dur - dur);
        }
        Ok(())
    }
    pub async fn cancel(&mut self) -> Result<()> {
        self.cancelled.store(true, Ordering::Relaxed);
        self.durations.lock().await.clear();
        Ok(()) // keep API similar with std tread
    }
}
