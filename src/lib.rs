//! This crate manages CPU affinities.
//! 
//! ## Example
//! 
//! This example shows how create a thread for each available processor and pin each thread to its corresponding processor. 
//! 
//! ```
//! extern crate core_affinity;
//! 
//! use std::thread;
//! 
//! // Retrieve the IDs of all active CPU cores.
//! let core_ids = core_affinity::get_core_ids().unwrap();
//! 
//! // Create a thread for each active CPU core.
//! let handles = core_ids.into_iter().map(|id| {
//!     thread::spawn(move || {
//!         // Pin this thread to a single CPU core.
//!         core_affinity::set_for_current(id);
//!         // Do more work after this.
//!     })
//! }).collect::<Vec<_>>();
//! 
//! for handle in handles.into_iter() {
//!     handle.join().unwrap();
//! }
//! ```

#[cfg(test)]
extern crate num_cpus;

/// This function tries to retrieve information
/// on all the "cores" active on this system.
pub fn get_core_ids() -> Option<Vec<CoreId>> {
    get_core_ids_helper()
}

/// This function tries to pin the current
/// thread to the specified core.
///
/// # Arguments
///
/// * core_id - ID of the core to pin
pub fn set_for_current(core_id: CoreId) {
    set_for_current_helper(core_id);
}

/// This represents a CPU core.
#[derive(Copy, Clone)]
pub struct CoreId {
    id: usize,
}

// Linux Section

#[cfg(target_os = "linux")]
#[inline]
fn get_core_ids_helper() -> Option<Vec<CoreId>> {
    linux::get_core_ids()
}

#[cfg(target_os = "linux")]
#[inline]
fn set_for_current_helper(core_id: CoreId) {
    linux::set_for_current(core_id);
}

#[cfg(target_os = "linux")]
extern crate libc;

#[cfg(target_os = "linux")]
mod linux {
    use std::mem;

    use libc;

    use super::CoreId;
    
    pub fn get_core_ids() -> Option<Vec<CoreId>> {
        if let Some(full_set) = get_affinity_mask() {
            let mut core_ids: Vec<CoreId> = Vec::new();

            for i in 0..libc::CPU_SETSIZE as usize {
                if unsafe { libc::CPU_ISSET(i, &full_set) } {
                    core_ids.push(CoreId{ id: i });
                }
            }

            Some(core_ids)
        }
        else {
            None
        }
    }

    pub fn set_for_current(core_id: CoreId) {
        // Turn `core_id` into a `libc::cpu_set_t` with only
        // one core active.
        let mut set = new_cpu_set();

        unsafe { libc::CPU_SET(core_id.id, &mut set) };

        // Set the current thread's core affinity.
        unsafe {
            libc::sched_setaffinity(0, // Defaults to current thread
                                    mem::size_of::<libc::cpu_set_t>(),
                                    &set);
        }
    }

    fn get_affinity_mask() -> Option<libc::cpu_set_t> {
        let mut set = new_cpu_set();

        // Try to get current core affinity mask.
        let result = unsafe {
            libc::sched_getaffinity(0, // Defaults to current thread
                                    mem::size_of::<libc::cpu_set_t>(),
                                    &mut set)
        };

        if result == 0 {
            Some(set)
        }
        else {
            None
        }
    }

    fn new_cpu_set() -> libc::cpu_set_t {
        unsafe { mem::zeroed::<libc::cpu_set_t>() }
    }

    #[cfg(test)]
    mod tests {
        use num_cpus;
        
        use super::*;
        
        #[test]
        fn test_linux_get_affinity_mask() {
            match get_affinity_mask() {
                Some(_) => {},
                None => { assert!(false); },
            }
        }
        
        #[test]
        fn test_linux_get_core_ids() {
            match get_core_ids() {
                Some(set) => {
                    assert_eq!(set.len(), num_cpus::get());
                },
                None => { assert!(false); },
            }
        }
        
        #[test]
        fn test_linux_set_for_current() {
            let ids = get_core_ids().unwrap();

            assert!(ids.len() > 0);

            set_for_current(ids[0]);

            // Ensure that the system pinned the current thread
            // to the specified core.
            let mut core_mask = new_cpu_set();
            unsafe { libc::CPU_SET(ids[0].id, &mut core_mask) };

            let new_mask = get_affinity_mask().unwrap();

            let mut is_equal = true;

            for i in 0..libc::CPU_SETSIZE as usize {
                let is_set1 = unsafe {
                    libc::CPU_ISSET(i, &core_mask)
                };
                let is_set2 = unsafe {
                    libc::CPU_ISSET(i, &new_mask)
                };

                if is_set1 != is_set2 {
                    is_equal = false;
                }
            }

            assert!(is_equal);
        }
     }
}

// Windows Section

#[cfg(target_os = "windows")]
#[inline]
fn get_core_ids_helper() -> Option<Vec<CoreId>> {
    windows::get_core_ids()
}

#[cfg(target_os = "windows")]
#[inline]
fn set_for_current_helper(core_id: CoreId) {
    windows::set_for_current(core_id);
}

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "windows")]
mod windows {
    use std::mem;

    use winapi;

    use super::CoreId;
    
    pub fn get_core_ids() -> Option<Vec<CoreId>> {
        if let Some(mask) = get_affinity_mask() {
            // Find all active cores in the bitmask.
            let mut core_ids: Vec<CoreId> = Vec::new();

            for i in 0..64 as u64 {
                let test_mask = 1 << i;

                if (mask & test_mask) == test_mask {
                    core_ids.push(CoreId { id: i as usize });
                }
            }

            Some(core_ids)
        }
        else {
            None
        }
    }

    pub fn set_for_current(core_id: CoreId) {
        // Convert `CoreId` back into mask.
        let mask: u64 = 1 << core_id.id;

        // Set core affinity for current thread.
        let res = unsafe {
            winapi::kernel32::SetThreadAffinityMask(
                winapi::kernel32::GetCurrentThread(),
                mask as winapi::basetsd::DWORD_PTR
            )
        };
    }

    fn get_affinity_mask() -> Option<u64> {
        let mut process_mask: u64 = 0;
        let mut system_mask: u64 = 0;

        let res = unsafe {
            winapi::kernel32::GetProcessAffinityMask(
                winapi::kernel32::GetCurrentProcess(),
                &process_mask as winapi::basetsd::PDWORD_PTR,
                &system_mask as winapi::basetsd::PDWORD_PTR
            )
        };

        // Successfully retrieved affinity mask
        if res != 0 {
            Some(process_mask)
        }
        // Failed to retrieve affinity mask
        else {
            None
        }
    }
}

// Other section
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[inline]
fn get_core_ids_helper() -> Option<Vec<CoreId>> {
    None
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[inline]
fn set_for_current_helper(core_id: CoreId) {
}

#[cfg(test)]
mod tests {
    use num_cpus;
    
    use super::*;

    #[test]
    fn test_num_cpus() {
        println!("Num CPUs: {}", num_cpus::get());
        println!("Num Physical CPUs: {}", num_cpus::get_physical());
    }
    
    #[test]
    fn test_get_core_ids() {
        match get_core_ids() {
            Some(set) => {
                assert_eq!(set.len(), num_cpus::get());
            },
            None => { assert!(false); },
        }
    }

    #[test]
    fn test_set_for_current() {
        let ids = get_core_ids().unwrap();

        assert!(ids.len() > 0);

        set_for_current(ids[0]);
    }
}
