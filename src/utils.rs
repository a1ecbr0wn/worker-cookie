//! Runtime utilities for Cloudflare Workers environment.

use cfg_if::cfg_if;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        /// Installs a panic hook that logs panic messages to the browser console.
        ///
        /// When the `console_error_panic_hook` feature is enabled, this configures
        /// the default panic handler to display human-readable panic messages in the
        /// worker's console instead of generating cryptic WebAssembly runtime errors.
        pub use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        /// Installs a panic hook that logs panic messages to the browser console.
        ///
        /// This is a no-op stub when the `console_error_panic_hook` feature is not enabled.
        #[inline]
        pub fn set_panic_hook() {}
    }
}
