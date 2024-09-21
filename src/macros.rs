#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

// Run a unit test on both wasm32 and non-wasm32 targets
macro_rules! cross_target_test {
    ($test:item) => {
        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen_test]
        $test

        #[cfg(not(target_arch = "wasm32"))]
        #[test]
        $test
    };
}