#!/bin/bash

truncate -s 100M dummy

./osbx decode dummy

echo ""

./osbx show dummy

echo ""

./osbx rescue dummy .

echo ""

./osbx rescue dummy . log

rm log
