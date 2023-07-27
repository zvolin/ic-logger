# ic-logger [![](https://img.shields.io/github/tag/borntyping/rust-simple_logger.svg)](https://github.com/borntyping/rust-simple_logger/tags) [![](https://img.shields.io/travis/borntyping/rust-simple_logger.svg)](https://travis-ci.org/borntyping/rust-simple_logger) [![](https://img.shields.io/github/issues/borntyping/rust-simple_logger.svg)](https://github.com/borntyping/rust-simple_logger/issues)

A simple logging backend for [ICP](https://internetcomputer.org) canisters.

Usage
-----

```rust
use ic_cdk::{init, query};

mod foo {
    pub fn bar() {
        log::warn!("sample log");
    }
}

#[init]
async fn init() {
    ic_cdk::setup();
}

#[update]
async fn baz() -> Result<()> {
    let _ = ic_logger::init();

    foo::bar();
}
```

This outputs:

```txt
2023-07-27 23:08:09.718590904 UTC: [Canister bkyz2-fmaaa-aaaaa-qaaaq-cai] [WARN  my_canister::foo] sample log
```

You can run the above example with:

```sh
dfx start --clean --background
dfx deploy
dfx call my_canister baz
```

As the canister's flexible memory may be dropped, it's suggested to call `ic_logger::init()` (or any equivalent)
in each canister function and drop the result in case the logger was already initialized.

Licence
-------

`ic-logger` is licenced under the [MIT Licence](http://opensource.org/licenses/MIT).

Credits
-------

Forked from [simple_logger](https://github.com/borntyping/rust-simple_logger) written by [Sam Clements](sam@borntyping.co.uk).
