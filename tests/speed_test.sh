#!/bin/bash

truncate -s 100M dummy

./rsbx decode dummy

echo ""

./rsbx show dummy

echo ""

./rsbx rescue dummy .

echo ""

./rsbx rescue dummy . log

rm log
