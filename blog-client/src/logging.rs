#[cfg(not(debug_assertions))]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(not(debug_assertions))]
pub fn setup_logger() {
    crate::logging::log("Is release mode. Logging disabled.");
}

#[cfg(debug_assertions)]
pub mod realtime_log_change {
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn set_log_level(level: usize) {
        const LEVELS: [log::LevelFilter; 5] = [
            log::LevelFilter::Error,
            log::LevelFilter::Warn,
            log::LevelFilter::Info,
            log::LevelFilter::Debug,
            log::LevelFilter::Trace,
        ];
        log::set_max_level(LEVELS[level]);
    }
}

#[cfg(debug_assertions)]
pub fn setup_logger() {
    fern::Dispatch::new()
        .chain(fern::Output::call(console_log::log))
        .apply()
        .unwrap()
}
