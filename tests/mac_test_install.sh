#!/bin/bash

# Uncomment the relevant line if needed
brew install opam                   # Homebrew, OSX Mavericks or later
# brew install opam --without-aspcud  # Homebrew, OSX Mountain Lion or lower
# port install opam                   # MacPort

# Pin the project
opam pin add osbx . -n

# Initial build
opam install osbx
