#!/bin/bash

truncate -s 50m dummy

./../blkar encode dummy --nometa -f

# ./../blkar encode dummy -f

./../blkar decode dummy.sbx dummy -f
