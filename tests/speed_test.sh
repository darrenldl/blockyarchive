#!/bin/bash

./copy_release.sh

truncate -s 100M dummy

./blkar_release encode --sbx-version 19 --rs-data 10 --rs-parity 2 dummy -f

./blkar_release decode dummy

echo ""

./blkar_release show dummy

echo ""

mkdir rescued_data &>/dev/null

./blkar_release rescue dummy rescued_data

echo ""

./blkar_release rescue dummy . log

rm log
