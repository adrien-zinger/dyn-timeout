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

/// Dynamic timeout, standard implementation with std::thread. Automaticcaly
/// join on drop.
/// # Example
/// ```
/// use std::time::Duration;
/// use dyn_timeout::std_thread::DynTimeout;
///
/// const TWENTY: Duration = Duration::from_millis(20);
///
/// let dyn_timeout = DynTimeout::new(TWENTY, || {
///    println!("after forty milliseconds");
/// });
/// dyn_timeout.add(TWENTY).unwrap();
/// ```
pub struct DynTimeout {
    thread: Option<JoinHandle<()>>,
    cancelled: Arc<AtomicBool>,
    durations: DurationVec,
}

impl DynTimeout {
    /// Create a new dynamic timeout in a new thread. Execute the callback
    /// function in the separated thread after a given duration.
    /// The created thread join automatically on drop timeout without dismiss
    /// the callback execution.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use dyn_timeout::std_thread::DynTimeout;
    ///
    /// const TWENTY: Duration = Duration::from_millis(20);
    ///
    /// let dyn_timeout = DynTimeout::new(TWENTY, || {
    ///    println!("after forty milliseconds");
    /// });
    /// dyn_timeout.add(TWENTY).unwrap();
    /// ```
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
    /// Increase the delay before the timeout.
    ///
    /// # Return
    /// Return a result with an error if the timeout already appened or it failed
    /// to increase the delay for any other reason.
    /// Otherwise it return an empty success.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use dyn_timeout::std_thread::DynTimeout;
    ///
    /// const TWENTY: Duration = Duration::from_millis(20);
    /// let dyn_timeout = DynTimeout::new(TWENTY, || {
    ///    println!("after forty milliseconds");
    /// });
    /// dyn_timeout.add(TWENTY).unwrap();
    /// ```
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
    /// Try to decrease the delay before the timeout. (work in progress)
    ///
    /// # Return
    /// Return a result with an error if the timeout already appened or it failed
    /// to decrease the delay for any other reason.
    /// Otherwise it return an empty success.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use dyn_timeout::std_thread::DynTimeout;
    ///
    /// const TWENTY: Duration = Duration::from_millis(20);
    /// const TEN: Duration = Duration::from_millis(10);
    ///
    /// let dyn_timeout = DynTimeout::new(TWENTY, || {
    ///    println!("after some milliseconds");
    /// });
    /// dyn_timeout.add(TEN).unwrap();
    /// dyn_timeout.add(TWENTY).unwrap();
    /// dyn_timeout.sub(TEN).unwrap();
    /// ```
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
    /// Dismiss the timeout callback and cancel all delays added.
    /// Join the created thread. (Note: we're
    /// currently working on a fast cancellation of all the delays)
    ///
    /// # Return
    /// Return a result with an error if the timeout if the program failed to
    /// clear the delays.
    /// Otherwise it return an empty success.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use dyn_timeout::std_thread::DynTimeout;
    ///
    /// const TWENTY: Duration = Duration::from_millis(20);
    /// const TEN: Duration = Duration::from_millis(10);
    ///
    /// let mut dyn_timeout = DynTimeout::new(TWENTY, || {
    ///    println!("never append");
    /// });
    /// dyn_timeout.add(TEN).unwrap();
    /// // cancel the last ten milliseconds and dismiss the callback
    /// dyn_timeout.cancel().unwrap();
    /// ```
    pub fn cancel(&mut self) -> Result<()> {
        match self.durations.lock() {
            Ok(mut durations) => {
                self.cancelled.store(true, Ordering::Relaxed);
                durations.clear()
            }
            Err(err) => bail!(err.to_string()),
        };
        self.join()?;
        self.thread = None;
        Ok(())
    }
    fn join(&mut self) -> Result<()> {
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
