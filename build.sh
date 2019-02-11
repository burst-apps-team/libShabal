#!/bin/bash

cargo build --release --features "simd"

cd target/release
zip ../../${TRAVIS_OS_NAME}.zip libshabal.*
