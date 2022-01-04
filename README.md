[![CI](https://github.com/adrien-zinger/dyn-timeout/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/adrien-zinger/dyn-timeout/actions/workflows/ci.yml?query=branch%3Amain)

# Dynamic Timeout

Execute a function after a mutable duration.

```rust
use std::time::Duration;
use dyn_timeout::std_thread::DynTimeout;

const TWENTY: Duration = Duration::from_millis(20);

let dyn_timeout = DynTimeout::new(TWENTY, || {
   println!("after forty milliseconds");
});
dyn_timeout.add(TWENTY).unwrap();
// .sub...
// .cancel
```

This library was initially implemented to be used as a [raft like election timeout](https://raft.github.io/).

## Tokio version

This crate include a std with threads and a tokio implementation, usefull if you're already using this async library.

```rust
use tokio::runtime::Runtime;
use dyn_timeout::tokio_impl::DynTimeout;
use std::time::Duration;
const TWENTY: Duration = Duration::from_millis(20);

let mut rt = Runtime::new().unwrap(); 
rt.spawn(async {
   let dyn_timeout = DynTimeout::new(TWENTY, || {
       println!("after forty milliseconds");
   });
   dyn_timeout.add(TWENTY).await.unwrap();
});
```

## Contribute

- All increases of the time precision and code architecture are welcomes.
- Usage examples, documentation, typo and comments, unit tests.
- All interestings ideas are also good to know in an issue.
- Give your feedback, report errors and bugs in a github issue.

All development contribution, please, has to pass the currents unit tests and it's a must to include a new test!

---
#### License GNU GENERAL PUBLIC LICENSE Version 3, 29 June 2007

<sub>This lirary is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.

