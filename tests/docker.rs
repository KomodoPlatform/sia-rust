#![feature(test)]
#![feature(custom_test_frameworks)]
#![test_runner(docker_tests_runner)]

#[path = "docker/mod.rs"] mod docker;

#[macro_use] extern crate lazy_static;
extern crate test;

use docker::utils::{SIA_DOCKER_IMAGE, SIA_DOCKER_IMAGE_WITH_TAG, SIA_WALLATD_RPC_PORT};

use std::env;
use std::io::{BufRead, BufReader};
use std::process::Command;
use test::{test_main, StaticBenchFn, StaticTestFn, TestDescAndFn};
use testcontainers::clients::Cli;
use testcontainers::{Container, GenericImage, RunnableImage};

/// Custom test runner intended to initialize the SIA coin daemon in a Docker container.
pub fn docker_tests_runner(tests: &[&TestDescAndFn]) {
    let docker = Cli::default();

    pull_docker_image(SIA_DOCKER_IMAGE_WITH_TAG);
    remove_docker_containers(SIA_DOCKER_IMAGE_WITH_TAG);
    let _sia_node = sia_docker_node(&docker, SIA_WALLATD_RPC_PORT);

    let owned_tests: Vec<_> = tests
        .iter()
        .map(|t| match t.testfn {
            StaticTestFn(f) => TestDescAndFn {
                testfn: StaticTestFn(f),
                desc: t.desc.clone(),
            },
            StaticBenchFn(f) => TestDescAndFn {
                testfn: StaticBenchFn(f),
                desc: t.desc.clone(),
            },
            _ => panic!("non-static tests passed to the test runner"),
        })
        .collect();

    let args: Vec<_> = env::args().collect();
    test_main(&args, owned_tests, None);
}

fn pull_docker_image(name: &str) {
    Command::new("docker")
        .arg("pull")
        .arg(name)
        .status()
        .expect("Failed to execute docker command");
}

fn remove_docker_containers(name: &str) {
    let stdout = Command::new("docker")
        .arg("ps")
        .arg("-f")
        .arg(format!("ancestor={}", name))
        .arg("-q")
        .output()
        .expect("Failed to execute docker command");

    let reader = BufReader::new(stdout.stdout.as_slice());
    let ids: Vec<_> = reader.lines().map(|line| line.unwrap()).collect();
    if !ids.is_empty() {
        Command::new("docker")
            .arg("rm")
            .arg("-f")
            .args(ids)
            .status()
            .expect("Failed to execute docker command");
    }
}

fn sia_docker_node(docker: &Cli, port: u16) -> Container<'_, GenericImage> {
    let image =
        GenericImage::new(SIA_DOCKER_IMAGE, "latest").with_env_var("WALLETD_API_PASSWORD", "password".to_string());
    let args = vec![];
    let image = RunnableImage::from((image, args))
        .with_mapped_port((port, port))
        .with_container_name("sia-docker");
    docker.run(image)
}
