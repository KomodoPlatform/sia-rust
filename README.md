# Sia Rust

`sia-rust` is a Rust implementation of Siacoin to be used primarily by the [Komodo DeFi Framework](https://github.com/KomodoPlatform/komodo-defi-framework). This crate provides the core functionalities to create and sign Siacoin transactions. 

## Features

- **V2 Transaction Builder**: Build Sia V2 transactions including SpendPolicy support
- **Walletd Client**: Interact with the Sia network via a local or remote instance of [Walletd](https://github.com/SiaFoundation/walletd).

## Requirements

Rust nightly-2023-06-01 is the only officially supported toolchain. This was chosen to keep this library inline with Komodo DeFi Framework. Similarly, dependencies have been locked to explicit versions to align with Komodo DeFi Framework's dependency tree.

## Contact

For any questions or suggestions, please open an issue on GitHub.

This project is supported by a [Sia Foundation grant](https://forum.sia.tech/t/standard-grant-proposal-htlc-upgrade-for-sia-for-use-in-atomic-swaps/410/5).

