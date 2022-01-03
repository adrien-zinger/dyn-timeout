#![feature(test)]
pub mod std_thread;
pub mod tokio_impl;

#[cfg(test)]
mod test {
    extern crate test;
    use std::time::Duration;
    use test::Bencher;
    const TWENTY: Duration = Duration::from_millis(20);
    use crate::std_thread;
    use crate::tokio_impl;

    #[test]
    fn simple_test() {
        let dyn_timeout = std_thread::DynTimeout::new(TWENTY, || {
            println!("after twenty nano second");
        });
        dyn_timeout.add(TWENTY).unwrap();
    }
    #[bench]
    fn simple_bench(b: &mut Bencher) {
        b.iter(|| {
            std_thread::DynTimeout::new(TWENTY, || {}).add(TWENTY).unwrap();
        });
    }
    #[tokio::test]
    async fn tokio_test() {
        let dyn_timeout = tokio_impl::DynTimeout::new(TWENTY, || {
            println!("after twenty nano second");
        });
        dyn_timeout.add(TWENTY).await.unwrap();
    }
}
