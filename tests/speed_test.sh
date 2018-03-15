#!/bin/bash

./copy_release.sh

truncate -s 100M dummy

./rsbx_release decode dummy

echo ""

./rsbx_release show dummy

echo ""

mkdir rescued_data &>/dev/null

./rsbx_release rescue dummy rescued_data

echo ""

./rsbx_release rescue dummy . log

rm log
