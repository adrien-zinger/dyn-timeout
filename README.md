# Dynamic Timeout

Execute a function after a mutable duration.

```rust
const TWENTY: Duration = Duration::from_millis(20);

let dyn_timeout = DynTimeout::new(TWENTY, || {
    println!("after fourty millis second");
});
dyn_timeout.add(TWENTY).unwrap();
// .sub...
// .cancel
```

This crate include a tokio and a standard implementation of the dynamic timeout.

---
## Contribute

- All increases of the timelaps precision are welcome.
- A system to cancel the timeout without waiting the last sleep (maybe using the crate [cancellable-timer](https://crates.io/crates/cancellable-timer))
- Usage examples and documentation


#### License

<sup>
Licensed under either of GNU GENERAL PUBLIC LICENSE Version 3, 29 June 2007
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
</sub>
