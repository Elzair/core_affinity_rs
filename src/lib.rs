#[cfg(target_os = "linux")]
extern crate libc;

use std::mem;

pub fn get_core_ids() -> Option<Vec<CoreId>> {
    #[cfg(target_os = "linux")]
    get_core_ids_linux()
}

pub fn set_for_current(core_id: CoreId) {
    #[cfg(target_os = "linux")]
    set_for_current_linux(core_id);
}

#[derive(Copy, Clone)]
pub struct CoreId {
    id: usize,
}

#[cfg(target_os = "linux")]
pub fn get_core_ids_linux() -> Option<Vec<CoreId>> {
    if let Some(full_set) = get_affinity_mask_linux() {
        let mut res: Vec<CoreId> = Vec::new();

        for i in 0..libc::CPU_SETSIZE as usize {
            if unsafe { libc::CPU_ISSET(i, &full_set) } {
                res.push(CoreId{ id: i });
            }
        }

        Some(res)
    }
    else {
        None
    }
}

pub fn set_for_current_linux(core_id: CoreId) {
    // Turn `core_id` into a `libc::cpu_set_t` with only
    // one core active.
    let mut set = new_cpu_set_linux();

    unsafe { libc::CPU_SET(core_id.id, &mut set) };

    // Set the current thread's core affinity.
    unsafe {
        libc::sched_setaffinity(0, // Defaults to current thread
                                mem::size_of::<libc::cpu_set_t>(),
                                &set);
    }
}

#[cfg(target_os = "linux")]
fn get_affinity_mask_linux() -> Option<libc::cpu_set_t> {
    let mut set = new_cpu_set_linux();

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

#[cfg(target_os = "linux")]
fn new_cpu_set_linux() -> libc::cpu_set_t {
    unsafe { mem::zeroed::<libc::cpu_set_t>() }
}

#[cfg(test)]
extern crate num_cpus;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_affinity_mask_linux() {
        match get_affinity_mask_linux() {
            Some(_) => {},
            None => { assert!(false); },
        }
    }
    
    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_core_ids_linux() {
        match get_core_ids_linux() {
            Some(set) => {
                assert_eq!(set.len(), num_cpus::get());
            },
            None => { assert!(false); },
        }
    }
    
    #[cfg(target_os = "linux")]
    #[test]
    fn test_set_for_current_linux() {
        let ids = get_core_ids_linux().unwrap();

        assert!(ids.len() > 0);

        set_for_current_linux(ids[0]);

        // Ensure that the system pinned the current thread
        // to the specified core.
        let mut core_mask = new_cpu_set_linux();
        unsafe { libc::CPU_SET(ids[0].id, &mut core_mask) };

        let new_mask = get_affinity_mask_linux().unwrap();

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
