#![feature(test)]
pub mod std_thread;
pub mod tokio;
#[cfg(test)]
mod test {
    extern crate test;
    use std::time::Duration;
    use test::Bencher;
    const TWENTY: Duration = Duration::from_millis(20);
    use crate::std_thread::DynTimeout;

    #[test]
    fn simple_test() {
        let dyn_timeout = DynTimeout::new(TWENTY, || {
            println!("after twenty nano second");
        });
        dyn_timeout.add(TWENTY).unwrap();
    }

    #[bench]
    fn simple_bench(b: &mut Bencher) {
        b.iter(|| {
            DynTimeout::new(TWENTY, || {}).add(TWENTY).unwrap();
        });
    }
}
