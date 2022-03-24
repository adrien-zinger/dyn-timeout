///! Implementation of the dynamic timeout using the tokio library
use anyhow::{bail, Result};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    task::JoinHandle,
};

type DurationVec = Arc<Mutex<Vec<Duration>>>;

/// Dynamic timeout, async implementation with the tokio library.
/// # Example
/// ```
/// use tokio::runtime::Runtime;
/// use dyn_timeout::tokio_impl::DynTimeout;
/// use std::time::Duration;
/// const TWENTY: Duration = Duration::from_millis(20);
///
/// let mut rt = Runtime::new().unwrap();
/// rt.spawn(async {
///    let dyn_timeout = DynTimeout::new(TWENTY, || {
///        println!("after forty milliseconds");
///    });
///    dyn_timeout.add(TWENTY).await.unwrap();
/// });
/// ```
pub struct DynTimeout {
    cancelled: Arc<AtomicBool>,
    durations: DurationVec,
    sender: mpsc::Sender<()>,
    thread: Option<JoinHandle<()>>,
    receiver: mpsc::Receiver<()>,
}

impl DynTimeout {
    /// Create a new dynamic timeout in a new thread. Execute the callback
    /// function in the separated thread after a given duration.
    ///
    /// # Example
    /// ```
    /// use tokio::runtime::Runtime;
    /// use dyn_timeout::tokio_impl::DynTimeout;
    /// use std::time::Duration;
    /// const TWENTY: Duration = Duration::from_millis(20);
    ///
    /// let mut rt = Runtime::new().unwrap();
    /// rt.spawn(async {
    ///    let dyn_timeout = DynTimeout::new(TWENTY, || {
    ///        println!("after forty milliseconds");
    ///    });
    ///    dyn_timeout.add(TWENTY).await.unwrap();
    /// });
    /// ```
    pub fn new(dur: Duration, callback: fn() -> ()) -> Self {
        let durations: DurationVec = Arc::new(Mutex::new(vec![Duration::ZERO, dur]));
        let thread_vec = durations.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let thread_cancelled = cancelled.clone();
        let (sender, mut receiver) = mpsc::channel::<()>(1);
        let (tx, rx) = mpsc::channel::<()>(1);
        Self {
            cancelled,
            durations,
            sender,
            receiver: rx,
            thread: Some(tokio::task::spawn(async move {
                while let Some(dur) = thread_vec.lock().await.pop() {
                    let _ = tokio::time::timeout(dur, async { receiver.recv().await }).await;
                }
                if !thread_cancelled.load(Ordering::Relaxed) {
                    //println!("hey");
                    callback();
                }
                tx.send(()).await.unwrap();
            })),
        }
    }
    /// Create a new dynamic timeout in a new thread. Call the mpsc sender on
    /// timeout reached.
    ///
    /// # Example
    /// ```
    /// use tokio::runtime::Runtime;
    /// use dyn_timeout::tokio_impl::DynTimeout;
    /// use std::time::Duration;
    /// const TWENTY: Duration = Duration::from_millis(20);
    ///
    /// let mut rt = Runtime::new().unwrap();
    /// rt.spawn(async {
    ///    let (sender, mut receiver) = tokio::sync::mpsc::channel::<()>(1);
    ///    let dyn_timeout = DynTimeout::with_sender(TWENTY, sender);
    ///    tokio::select! {
    ///     _ = receiver.recv() => println!("Timeout!")
    ///    }
    /// });
    /// ```
    pub fn with_sender(dur: Duration, sender_in: Sender<()>) -> Self {
        let durations: DurationVec = Arc::new(Mutex::new(vec![Duration::ZERO, dur]));
        let thread_vec = durations.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let thread_cancelled = cancelled.clone();
        let (sender, mut receiver) = mpsc::channel::<()>(1);
        let (tx, rx) = mpsc::channel::<()>(1);
        Self {
            cancelled,
            durations,
            sender,
            receiver: rx,
            thread: Some(tokio::task::spawn(async move {
                while let Some(dur) = thread_vec.lock().await.pop() {
                    let _ = tokio::time::timeout(dur, async { receiver.recv().await }).await;
                }
                if !thread_cancelled.load(Ordering::Relaxed) {
                    sender_in.send(()).await.unwrap();
                }
                tx.send(()).await.unwrap();
            })),
        }
    }
    /// Increase the delay before the timeout.
    ///
    /// # Return
    /// Return a result with an error if the timeout already appened.
    /// Otherwise it return an empty success.
    ///
    /// # Example
    /// ```
    /// use tokio::runtime::Runtime;
    /// use dyn_timeout::tokio_impl::DynTimeout;
    /// use std::time::Duration;
    /// const TWENTY: Duration = Duration::from_millis(20);
    ///
    /// let mut rt = Runtime::new().unwrap();
    /// rt.spawn(async {
    ///    let dyn_timeout = DynTimeout::new(TWENTY, || {
    ///        println!("after some milliseconds");
    ///    });
    ///    dyn_timeout.add(TWENTY).await.unwrap();
    /// });
    /// ```
    pub async fn add(&self, dur: Duration) -> Result<()> {
        let mut durations = self.durations.lock().await;
        if durations.is_empty() {
            bail!("Timeout already reached")
        }
        durations.push(dur);
        Ok(())
    }
    /// Try to decrease the delay before the timeout. (bad precision, work in progress)
    ///
    /// # Return
    /// Return a result with an error if the timeout already appened.
    /// Otherwise it return an empty success.
    ///
    /// # Example
    /// ```
    /// use tokio::runtime::Runtime;
    /// use dyn_timeout::tokio_impl::DynTimeout;
    /// use std::time::Duration;
    ///
    /// const TWENTY: Duration = Duration::from_millis(20);
    /// const TEN: Duration = Duration::from_millis(10);
    ///
    /// let mut rt = Runtime::new().unwrap();
    /// rt.spawn(async {
    ///    let dyn_timeout = DynTimeout::new(TWENTY, || {
    ///        println!("after some milliseconds");
    ///    });
    ///    dyn_timeout.add(TEN).await.unwrap();
    ///    dyn_timeout.add(TWENTY).await.unwrap();
    ///    dyn_timeout.sub(TEN).await.unwrap();
    /// });
    /// ```
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
    /// Dismiss the timeout callback and cancel all delays added.
    /// Stop immediatelly all waiting process and join the created thread.
    ///
    /// # Return
    /// Return a result with an error if the timeout already appened.
    /// Otherwise it return an empty success.
    ///
    /// # Example
    /// ```
    /// use tokio::runtime::Runtime;
    /// use dyn_timeout::tokio_impl::DynTimeout;
    /// use std::time::Duration;
    ///
    /// const TWENTY: Duration = Duration::from_millis(20);
    /// const TEN: Duration = Duration::from_millis(10);
    ///
    /// let mut rt = Runtime::new().unwrap();
    /// rt.spawn(async {
    ///    let mut dyn_timeout = DynTimeout::new(TWENTY, || {
    ///        println!("never append");
    ///    });
    ///    dyn_timeout.cancel().await.unwrap();
    /// });
    /// ```
    pub async fn cancel(&mut self) -> Result<()> {
        self.cancelled.store(true, Ordering::Relaxed);
        self.durations.lock().await.clear();
        self.sender.send(()).await?;
        self.thread = None;
        Ok(())
    }

    /// Wait for the end of the timeout
    pub async fn wait(&mut self) -> Result<()> {
        self.receiver.recv().await;
        Ok(())
    }
}
