#![feature(doc_cfg)]

// clippy doesn't see the cfg(doc) version, and adding docs to the others would
// likely lead to issues with macro_rules transcriber repeater nesting limitations
#![allow(clippy::missing_safety_doc)]

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
    /// Only kills the thread if it has enabled cancellation, then performs cleanup.
    /// See `man pthread_cancel` for more information.
    /// 
    /// # Safety
    ///
    /// See `man pthread_cancel`.
    
    decl: [pub unsafe fn kill_thread_graceful<T>(handle: JoinHandle<T>)]
    
    if unix {
        use std::os::unix::thread::JoinHandleExt;
        use libc::pthread_cancel;

        let raw_handle = handle.into_pthread_t();
        pthread_cancel(raw_handle);
    }
}

cfg_doc! {
    /// Forcibly and immediately stops the thread, without any cleanup.
    /// 
    /// # Safety
    ///
    /// See <https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminatethread>.
    
    decl: [pub unsafe fn kill_thread_forcibly_exit_code<T>(handle: JoinHandle<T>, exit_code: u32)]
    
    if windows {
        use std::os::windows::io::IntoRawHandle;
        use winapi::um::processthreadsapi::TerminateThread;
        use winapi::ctypes::c_void as winapi_c_void;

        let raw_handle = handle.into_raw_handle();
        TerminateThread(raw_handle as *mut winapi_c_void, exit_code);
    }
}

cfg_doc! {
    /// Suspends the thread.
    /// 
    /// # Safety
    /// 
    /// See <https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-suspendthread>.
    
    decl: [pub unsafe fn suspend_thread<T>(handle: JoinHandle<T>)] 
    
    if windows {
        use std::os::windows::io::IntoRawHandle;
        use winapi::um::processthreadsapi::SuspendThread;
        use winapi::ctypes::c_void as winapi_c_void;
        
        let raw_handle = handle.into_raw_handle();
        SuspendThread(raw_handle as *mut winapi_c_void);
    }
}
