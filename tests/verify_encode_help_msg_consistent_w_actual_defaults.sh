#!/bin/bash

exit_code=0

echo "Extracting defaults from encode help message"

defaults_from_help_msg=$(./../blkar encode --help | grep "Details of default option")
default_ver_from_help_msg=$(echo $defaults_from_help_msg | awk '{ print $6 }' | tr '=,' ' ' | awk '{ print $2 }')
default_rs_data_from_help_msg=$(echo $defaults_from_help_msg | awk '{ print $7 }' | tr '=,' ' ' | awk '{ print $2 }')
default_rs_parity_from_help_msg=$(echo $defaults_from_help_msg | awk '{ print $8 }' | tr '=,' ' ' | awk '{ print $2 }')
default_burst_from_help_msg=$(echo $defaults_from_help_msg | awk '{ print $9 }' | tr '=,' ' ' | awk '{ print $2 }')

echo "Encoding with default setting"

output=$(./../blkar encode --json -f dummy)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi

echo "Grabbing container information"
output=$(./../blkar show --guess-burst --json dummy.ecsbx)

echo -n "Checking it matches with actual defaults"
if [[ $(echo $output | jq -r ".blocks[0].sbxContainerVersion") == $default_ver_from_help_msg ]]; then
    echo -n " ==> Okay"
else
    echo -n " ==> NOT okay"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".blocks[0].rsDataShardCount") == $default_rs_data_from_help_msg ]]; then
    echo -n " ==> Okay"
else
    echo -n " ==> NOT okay"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".blocks[0].rsParityShardCount") == $default_rs_parity_from_help_msg ]]; then
    echo -n " ==> Okay"
else
    echo -n " ==> NOT okay"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".bestGuessForBurstErrorResistanceLevel") == $default_burst_from_help_msg ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

echo $exit_code > exit_code
