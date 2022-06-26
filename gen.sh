#!/bin/sh
cargo build
doas cp target/debug/odo .
doas chown root:root odo
doas chmod +s odo
