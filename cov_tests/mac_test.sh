#!/bin/bash

# Sync from repo and/or run initial install as needed prior to running.
# (see mac_test_install.sh)

# change to version needed
opam switch 4.04.2
eval `opam config env`

# Pin the project
cd ..
opam pin add rsbx . -n

# Build
opam uninstall rsbx
opam install rsbx

# Verify version
rsbx --version
