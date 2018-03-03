#!/bin/bash

truncate -s 50m dummy

./osbx encode dummy --nometa -f

# ./osbx encode dummy -f

./osbx decode dummy.sbx dummy -f
