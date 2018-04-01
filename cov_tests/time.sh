#!/bin/bash

/usr/bin/time -f "Elapsed time : %E, Peak Memory : %M, Waits : %w" ${@}
