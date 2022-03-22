pub mod std_thread;
pub mod tokio_impl;

#[cfg(test)]
mod test {
    //extern crate test;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime};
    //use test::Bencher;
    const TWENTY: Duration = Duration::from_millis(20);
    use crate::std_thread;
    use crate::tokio_impl;

    #[test]
    fn simple_test() {
        let dyn_timeout = std_thread::DynTimeout::new(TWENTY, || {
            println!("after forty milliseconds");
        });
        dyn_timeout.add(TWENTY).unwrap();
    }
    #[test]
    fn cancel_test() {
        let mut dyn_timeout = std_thread::DynTimeout::new(Duration::from_secs(20), || {
            panic!("Should never append");
        });
        dyn_timeout.add(Duration::from_secs(20)).unwrap();
        // this should be cancelled
        dyn_timeout.cancel().unwrap();
    }
    //#[bench]
    //fn simple_bench(b: &mut Bencher) {
    //    b.iter(|| {
    //        std_thread::DynTimeout::new(TWENTY, || {})
    //            .add(TWENTY)
    //            .unwrap();
    //    });
    //}
    #[tokio::test]
    async fn tokio_test() {
        let dyn_timeout = tokio_impl::DynTimeout::new(TWENTY, || {
            println!("after forty milliseconds");
        });
        dyn_timeout.add(TWENTY).await.unwrap();
    }

    lazy_static::lazy_static! {
        static ref TIME: Arc::<Mutex::<SystemTime>> = Arc::new(Mutex::new(SystemTime::now()));
    }

    #[tokio::test]
    async fn tokio_test_bench() {
        {
            let mut time = TIME.lock().unwrap();
            *time = SystemTime::now();
        }
        let mut dyn_timeout = tokio_impl::DynTimeout::new(TWENTY, move || {
            let st = TIME.lock().unwrap();
            let dur = st.elapsed().unwrap();
            assert!(dur > Duration::from_millis(36) && dur < Duration::from_millis(44));
        });
        dyn_timeout.add(TWENTY).await.unwrap();
        dyn_timeout.wait().await.unwrap();
    }
}
