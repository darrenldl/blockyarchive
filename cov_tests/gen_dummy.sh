#!/bin/bash
dd if=/dev/urandom of=dummy bs=$[1024 * 2] count=1 &>/dev/null
