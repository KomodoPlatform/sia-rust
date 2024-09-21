// Run a block of unit tests on both wasm32 and non-wasm32 targets
#[cfg(test)]
macro_rules! cross_target_tests {
    ($($test_fn:item)*) => {
        #[cfg(all(test, target_arch = "wasm32"))]
        use wasm_bindgen_test::*;

        #[cfg(all(test, target_arch = "wasm32"))]
        wasm_bindgen_test_configure!(run_in_browser);

        $(
            #[cfg(all(test, target_arch = "wasm32"))]
            #[wasm_bindgen_test]
            $test_fn

            #[cfg(not(target_arch = "wasm32"))]
            #[test]
            $test_fn
        )*
    };
}
