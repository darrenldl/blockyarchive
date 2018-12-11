#!/bin/bash

truncate -s 50m dummy

kcov_rsbx encode dummy --nometa -f

# kcov_rsbx encode dummy -f

kcov_rsbx decode dummy.sbx dummy -f
