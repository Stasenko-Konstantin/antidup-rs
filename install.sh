#!/bin/sh

cargo build --release
sudo cp target/release/antidup-rs /bin/antidup