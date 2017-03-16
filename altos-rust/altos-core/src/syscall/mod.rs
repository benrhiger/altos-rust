/*
* Copyright (C) 2017 AltOS-Rust Team
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

//! Syscall interface for the AltOS-Rust kernel.

use sched::{CURRENT_TASK, SLEEP_QUEUE, DELAY_QUEUE, OVERFLOW_DELAY_QUEUE, PRIORITY_QUEUES};
use task::{Delay, State, Priority};
use task::args::Args;
use task::{TaskHandle, TaskControl};
use queue::Node;
use alloc::boxed::Box;
use tick;
use sync::{MutexGuard, CondVar, CriticalSection};
use arch;
use atomic::{AtomicBool, Ordering};

// FIXME: When we can guarantee that syscalls will be executed in an interrupt free context, get
// rid of critical sections in this file.

/// An alias for the channel to sleep on that will never be awoken by a wakeup signal. It will
/// still be woken after a timeout.
pub const FOREVER_CHAN: usize = 0;

/// Creates a new task and puts it into the task queue for running. It returns a `TaskHandle`
/// which is used to monitor the task.
///
/// `new_task` takes several arguments, a `fn(&mut Args)` pointer which specifies the code to run
/// for the task, an `Args` argument for the arguments that will be passed to the task, a `usize`
/// argument for how much space should be allocated for the task's stack, a `Priority` argument for
/// the priority that the task should run with, and a `&str` argument to give the task a readable
/// name.
///
/// # Examples
///
/// ```rust,no_run
/// use altos_core::{start_scheduler, Priority};
/// use altos_core::syscall::new_task;
/// use altos_core::args::Args;
///
/// // Create the task and hold onto the handle
/// let handle = new_task(test_task, Args::empty(), 512, Priority::Normal, "new_task_name");
///
/// // Start running the task
/// start_scheduler();
///
/// fn test_task(_args: &mut Args) {
///   // Do stuff here...
///   loop {}
/// }
/// ```
pub fn new_task(code: fn(&mut Args), args: Args, stack_depth: usize, priority: Priority, name: &'static str)
    -> TaskHandle {

    // Make sure the task is allocated in one fell swoop
    let g = CriticalSection::begin();
    let task = Box::new(Node::new(TaskControl::new(code, args, stack_depth, priority, name)));
    drop(g);

    let handle = TaskHandle::new(&**task);
    PRIORITY_QUEUES[task.priority()].enqueue(task);
    handle
}

/// Exits and destroys the currently running task.
///
/// This function must only be called from within task code. Doing so from elsewhere (like an
/// interrupt handler, for example) will still destroy the currently running task. Since something
/// like an interrupt handler can interrupt any task, there's no way to determine which task it
/// would destroy.
///
/// It marks the currently running task to be destroyed, then immediatly yields to the scheduler
/// to allow another task to run.
///
/// # Examples
///
/// ```rust,no_run
/// use altos_core::syscall;
///
/// fn test_task(_args: &mut Args) {
///   // Do some stuff
///
///   syscall::exit();
/// }
/// ```
///
/// # Panics
///
/// This function will panic if the task is not successfully destroyed (i.e. it gets scheduled
/// after this function is called), but this should never happen.
pub fn exit() -> ! {
    // UNSAFE: This can only be called from the currently running task, so we know we're the only
    // one with a reference to the task. The destroy method is atomic so we don't have to worry
    // about any threading issues.
    unsafe {
        debug_assert!(CURRENT_TASK.is_some());
        CURRENT_TASK.as_mut().unwrap().destroy();
    }
    sched_yield();
    panic!("syscall::exit - task returned from exit!");
}

/// Yield the current task to the scheduler so another task can run.
///
/// # Examples
///
/// ```rust,no_run
/// use altos_core::syscall::sched_yield;
/// use altos_core::args::Args;
///
/// fn test_task(_args: &mut Args) {
///   loop {
///     // Do some important work...
///
///     // Okay, we're done...
///     sched_yield();
///     // Go back and do it again
///   }
/// }
/// ```
pub fn sched_yield() {
    arch::yield_cpu();
}

/// Put the current task to sleep, waiting on a channel to be woken up.
///
/// `sleep` takes a `usize` argument that acts as an identifier for when to wake up the task. The
/// task will sleep indefinitely if no wakeup signal is sent.
///
/// # Examples
///
/// ```no_run
/// use altos_core::syscall::sleep;
/// use altos_core::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
///
/// static flag: AtomicBool = ATOMIC_BOOL_INIT;
///
/// while !flag.load(Ordering::SeqCst) {
///   // Block until some other thread wakes us up
///   sleep(&flag as *const _ as usize);
/// }
/// ```
pub fn sleep(wchan: usize) {
    debug_assert!(wchan != FOREVER_CHAN);
    // Make the critical section for the whole function, wouldn't want to be rude and make a task
    // give up its time slice for no reason
    let _g = CriticalSection::begin();
    // UNSAFE: Accessing CURRENT_TASK
    match unsafe { CURRENT_TASK.as_mut() } {
        Some(current) => current.sleep(wchan),
        None => panic!("sleep - current task doesn't exist!"),
    }
    sched_yield();
}

/// Put the current task to sleep with a timeout, waiting on a channel to be woken up.
///
/// `sleep_for` takes a `usize` argument that acts as an identifier to wake up the task. It also
/// takes a second `usize` argument for the maximum ticks it should sleep before waking.
///
/// # Examples
///
/// ```no_run
/// use altos_core::syscall::{sleep_for, FOREVER_CHAN};
///
/// // Sleep for 300 ticks
/// sleep_for(FOREVER_CHAN, 300);
/// ```
pub fn sleep_for(wchan: usize, delay: usize) {
    // Make the critical section for the whole function, wouldn't want to be rude and make a task
    // give up its time slice for no reason
    let _g = CriticalSection::begin();
    // UNSAFE: Accessing CURRENT_TASK
    match unsafe { CURRENT_TASK.as_mut() } {
        Some(current) => current.sleep_for(wchan, delay),
        None => panic!("sleep_for - current task doesn't exist!"),
    }
    sched_yield();
}

/// Wake up all tasks sleeping on a channel.
///
/// `wake` takes a `usize` argument that acts as an identifier. This will wake up any tasks
/// sleeping on that identifier.
pub fn wake(wchan: usize) {
    // Since we're messing around with all the task queues, lets make sure everything gets done at
    // once
    let _g = CriticalSection::begin();
    let mut to_wake = SLEEP_QUEUE.remove(|task| task.wchan() == wchan);
    to_wake.append(DELAY_QUEUE.remove(|task| task.wchan() == wchan));
    to_wake.append(OVERFLOW_DELAY_QUEUE.remove(|task| task.wchan() == wchan));
    for mut task in to_wake {
        task.wake();
        PRIORITY_QUEUES[task.priority()].enqueue(task);
    }
}

/// Update the system tick count and wake up any delayed tasks that need to be woken.
///
/// This function will wake any tasks that have a delay.
#[doc(hidden)]
pub fn system_tick() {
    debug_assert!(arch::in_kernel_mode());

    // TODO: Do we need a critical section here? We should be in the tick handler
    let _g = CriticalSection::begin();
    tick::tick();

    // wake up all tasks sleeping until the current tick
    let ticks = tick::get_tick();

    let to_wake = DELAY_QUEUE.remove(|task| task.tick_to_wake() <= ticks);
    for mut task in to_wake {
        task.wake();
        PRIORITY_QUEUES[task.priority()].enqueue(task);
    }

    // If ticks == all 1's then it's about to overflow.
    if ticks == !0 {
        let overflowed = OVERFLOW_DELAY_QUEUE.remove_all();
        DELAY_QUEUE.append(overflowed);
    }

    // UNSAFE: Accessing CURRENT_TASK
    let current_priority = unsafe {
        match CURRENT_TASK.as_ref() {
            Some(task) => task.priority(),
            None => panic!("system_tick - current task doesn't exist!"),
        }
    };

    for i in Priority::higher(current_priority) {
        if !PRIORITY_QUEUES[i].is_empty() {
            // Only context switch if there's another task at the same or higher priority level
            sched_yield();
            break;
        }
    }
}

pub fn mutex_lock(lock: &AtomicBool) {
    while lock.compare_and_swap(false, true, Ordering::Acquire) != false {
        // let another process run if we can't get the lock
        let wchan = lock as *const _ as usize;
        sleep(wchan);
    }
}

pub fn mutex_unlock(lock: &AtomicBool) {
    lock.store(false, Ordering::Release);
    let wchan = lock as *const _ as usize;
    wake(wchan);
}

pub fn condvar_wait<'a, T>(condvar: &CondVar, guard: MutexGuard<'a, T>) {
    let _g = CriticalSection::begin();
    sleep(condvar as *const _ as usize);
    // Explicitly drop guard so that the mutex is unlocked before context switching.
    drop(guard);
}

#[cfg(test)]
mod tests {
    use test;
    use super::*;
    use task::args::Args;
    use sched::start_scheduler;

    #[test]
    fn test_new_task() {
        let _g = test::set_up();
        let handle = new_task(test_task, Args::empty(), 512, Priority::Normal, "test creation task");
        assert_eq!(handle.name(), Ok("test creation task"));
        assert_eq!(handle.priority(), Ok(Priority::Normal));
        assert_eq!(handle.state(), Ok(State::Ready));
        assert_eq!(handle.stack_size(), Ok(512));

        assert_not!(PRIORITY_QUEUES[Priority::Normal].remove_all().is_empty());
    }

    #[test]
    fn test_sched_yield() {
        // This isn't the greatest test, as the functionality of this method is really just
        // dependent on the platform implementation, but at least we can make sure it's working
        // properly for the test suite
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        sched_yield();
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));
    }

    #[test]
    fn test_sleep() {
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        // There's some special logic when something sleeps on FOREVER_CHAN, so make sure we don't
        // sleep on it
        sleep(!FOREVER_CHAN);
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        sched_yield();
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        sched_yield();
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));
    }

    #[test]
    fn test_wake() {
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        // There's some special logic when something sleeps on FOREVER_CHAN, so make sure we don't
        // sleep on it
        sleep(!FOREVER_CHAN);
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        sched_yield();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));


        wake(!FOREVER_CHAN);
        assert_ne!(handle_1.state(), Ok(State::Blocked));
        // wake should NOT yield the task, so we should still be running task 2
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        sched_yield();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));
    }

    #[test]
    fn test_system_tick() {
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        let old_tick = tick::get_tick();
        system_tick();
        assert_eq!(old_tick + 1, tick::get_tick());
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));
    }

    #[test]
    fn test_sleep_for_forever() {
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        sleep_for(FOREVER_CHAN, 4);
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        // 4 Ticks have passed, task 1 should be woken up now
        assert_ne!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));
    }

    #[test]
    fn test_sleep_for_timeout() {
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        sleep_for(!FOREVER_CHAN, 4);
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        // 4 Ticks have passed, task 1 should be woken up now
        assert_ne!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));
    }

    #[test]
    fn test_sleep_for_early_wake() {
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        sleep_for(!FOREVER_CHAN, 4);
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        wake(!FOREVER_CHAN);
        assert_ne!(handle_1.state(), Ok(State::Blocked));
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));
    }

    #[test]
    fn test_sleep_for_no_timeout_forever() {
        let _g = test::set_up();
        let (handle_1, handle_2) = test::create_two_tasks();

        start_scheduler();
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));

        // This should yield the task but immediately wake up on the next tick
        sleep_for(FOREVER_CHAN, 0);
        assert_eq!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_2.tid(), Ok(test::current_task().unwrap().tid()));

        system_tick();
        assert_ne!(handle_1.state(), Ok(State::Blocked));
        assert!(test::current_task().is_some());
        assert_eq!(handle_1.tid(), Ok(test::current_task().unwrap().tid()));
    }

    // Stub used for new_task calls.
    fn test_task(_args: &mut Args) {}
}
