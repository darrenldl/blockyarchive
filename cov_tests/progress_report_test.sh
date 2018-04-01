#!/bin/bash

truncate -s 50m dummy

./rsbx encode dummy --nometa -f

# ./rsbx encode dummy -f

./rsbx decode dummy.sbx dummy -f
