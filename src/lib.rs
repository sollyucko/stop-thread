#![feature(doc_cfg)]

// #![feature(trace_macros)]
// trace_macros!(true);

use std::thread::JoinHandle;

macro_rules! unbracket {
    (_ [$($tt1:tt)*]) => { $($tt1)* };
    (() [$($tt1:tt)*]) => { ($($tt1)*) };
    ([] [$($tt1:tt)*]) => { [$($tt1)*] };
    ({} [$($tt1:tt)*]) => { {$($tt1)*} };
    ($tt0:tt [$($tt1:tt)*] @unbracket ($($tt2:tt)*) $($tt3:tt)*) => { unbracket!{ $tt0 [$($tt1)* $($tt2)*] $($tt3)*} };
    ($tt0:tt [$($tt1:tt)*] @unbracket [$($tt2:tt)*] $($tt3:tt)*) => { unbracket!{ $tt0 [$($tt1)* $($tt2)*] $($tt3)*} };
    ($tt0:tt [$($tt1:tt)*] @unbracket {$($tt2:tt)*} $($tt3:tt)*) => { unbracket!{ $tt0 [$($tt1)* $($tt2)*] $($tt3)*} };
    ($tt0:tt [$($tt1:tt)*] $tt2:tt $($tt3:tt)*) => { unbracket!{$tt0 [$($tt1)* $tt2] $($tt3)*} };
}

macro_rules! cfg_doc {
    ($(#[doc=$doc0:literal])+ decl: $decl:tt $($($(#[doc=$doc1:literal])+)? if $cfg:tt $body:tt)*) => {
        unbracket! {_ []
            #[cfg(doc)]
            #[doc(cfg(any($($cfg),*)))]
            $(#[doc=$doc0])+
            $($(
                $(#[doc=$doc1])+
            )?)*
            @unbracket $decl {
                unimplemented!("Documentation implementation!");
            }

            $(
                #[cfg(all(not(doc), $cfg))]
                @unbracket $decl
                    $body
            )*
        }
    }
}

cfg_doc! {
    /// Kills the thread using `pthread_cancel` or `TerminateThread`.

    decl: [pub unsafe fn kill_thread<T>(handle: JoinHandle<T>)]

    /// # Unix
    ///
    /// ## Safety
    ///
    /// Only kills the thread if it has enabled cancellation, then performs cleanup.
    /// See `man pthread_cancel` for more information.
    if unix {
        #![allow(clippy::missing_safety_doc)]

        use std::os::unix::thread::JoinHandleExt;
        use libc::pthread_cancel;

        let raw_handle = handle.into_pthread_t();
        pthread_cancel(raw_handle);
    }

    /// # Windows
    ///
    /// Uses u32::MAX as the exit code.
    ///
    /// ## Safety
    ///
    /// Forcibly and immediately stops the thread, without any cleanup.
    /// See <https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminatethread>
    /// for more information.
    if windows {
        #![allow(clippy::missing_safety_doc)]

        kill_thread_exit_code(handle, u32::MAX);
    }
}

cfg_doc! {
    /// Kills the thread, specifying the thread's exit code (if applicable).

    decl: [pub unsafe fn kill_thread_exit_code<T>(handle: JoinHandle<T>, exit_code: u32)]

    /// # Unix
    ///
    /// ## Safety
    ///
    /// Only kills the thread if it has enabled cancellation, then performs cleanup.
    /// See `man pthread_cancel` for more information.
    if unix {
        #![allow(unused_variables)]
        #![allow(clippy::missing_safety_doc)]

        kill_thread(handle);
    }

    /// # Windows
    ///
    /// ## Safety
    ///
    /// Forcibly and immediately stops the thread, without any cleanup.
    /// See <https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminatethread>
    /// for more information.
    if windows {
        #![allow(clippy::missing_safety_doc)]

        use std::os::windows::io::IntoRawHandle;
        use winapi::um::processthreadsapi::TerminateThread;
        use winapi::ctypes::c_void as winapi_c_void;

        let raw_handle = handle.into_raw_handle();
        TerminateThread(raw_handle as *mut winapi_c_void, exit_code);
    }
}
