#!/bin/bash

cd ..

echo "Building osbx"
jbuilder build @install
echo ""

echo "Copying osbx binary over"
cp _build/default/src/osbx.exe ./tests/osbx
echo ""

cd tests


