use sia_rust::types::Address;
use std::future::Future;
use std::process::Command;

pub const SIA_DOCKER_IMAGE: &str = "docker.io/alrighttt/walletd-komodo";
pub const SIA_DOCKER_IMAGE_WITH_TAG: &str = "docker.io/alrighttt/walletd-komodo:latest";
pub const SIA_WALLATD_RPC_PORT: u16 = 9980;
pub const SIA_WALLETD_RPC_URL: &str = "http://localhost:9980/";

pub fn mine_blocks(n: u64, addr: &Address) {
    Command::new("docker")
        .arg("exec")
        .arg("sia-docker")
        .arg("walletd")
        .arg("mine")
        .arg(format!("-addr={}", addr))
        .arg(format!("-n={}", n))
        .status()
        .expect("Failed to execute docker command");
}

pub fn block_on<F>(fut: F) -> F::Output
where
    F: Future,
{
    #[cfg(not(target = "wasm32"))]
    {
        lazy_static! {
            pub static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
        }
        RUNTIME.block_on(fut)
    }
    // Not actually needed since we don't run end-to-end tests for wasm.
    // TODO: Generalize over the construction of platform-specific objects and use
    //       #[wasm_bindgen_test(unsupported = test)] macro to test both wasm and non-wasm targets
    #[cfg(target = "wasm32")]
    futures::executor::block_on(fut)
}
