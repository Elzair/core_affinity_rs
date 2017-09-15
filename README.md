`core-affinity-rs` is a Rust crate for managing CPU affinities. It currently supports Windows and Linux.

## Example

This example shows how create a thread for each available processor and pin each thread to its corresponding processor. 

```
extern crate core_affinity;

use std::thread;

// Retrieve the IDs of all active CPU cores.
let core_ids = core_affinity::get_core_ids().unwrap();

// Create a thread for each active CPU core.
let handles = core_ids.into_iter().map(|id| {
    thread::spawn(move || {
        // Pin this thread to a single CPU core.
        core_affinity::set_for_current(id);
        // Do more work after this.
    })
}).collect::<Vec<_>>();

for handle in handles.into_iter() {
    handle.join().unwrap();
}
```
