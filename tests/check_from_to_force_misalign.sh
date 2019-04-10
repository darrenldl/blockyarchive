#!/bin/bash

exit_code=0

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

offset=$[1 + RANDOM % 100]

echo -n "Encoding"
output=$(./../blkar encode --json -f dummy --uid DEADBEEF0001)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.fileUID") == "DEADBEEF0001" ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

mv dummy.ecsbx dummy.ecsbx.tmp
touch dummy.ecsbx
truncate -s $offset dummy.ecsbx
cat dummy.ecsbx.tmp >> dummy.ecsbx
rm dummy.ecsbx.tmp

echo "Checking dummy disk"

echo "Collecting base statistics"
output=$(./../blkar check --json --from $offset --force-misalign dummy.ecsbx)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
blocks_processed=$(echo $output | jq -r ".stats.numberOfBlocksProcessed")
okay_meta=$(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata")
okay_data=$(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData")

echo -n "Checking that blkar only checks the first block"
output=$(./../blkar check --json dummy.ecsbx --from $offset --to-inc $[offset + 511] --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

corrupt $[offset + 0] dummy.ecsbx

output=$(./../blkar check --json dummy.ecsbx --from $offset --to-inc $[offset + 511] --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 1 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking that blkar only checks the second block"
output=$(./../blkar check --json dummy.ecsbx --from $[offset + 512] --to-inc $[offset + 512] --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

corrupt $[offset + 512] dummy.ecsbx

output=$(./../blkar check --json dummy.ecsbx --from $[offset + 512] --to-inc $[offset + 512] --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 1 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking that blkar checks both blocks"
output=$(./../blkar check --json dummy.ecsbx --from $[offset + 0] --to-exc $[offset + 1024] --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 2 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 2 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking that blkar checks all blocks"
output=$(./../blkar check --json dummy.ecsbx --from $offset --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == $blocks_processed ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == $[okay_meta - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == $[okay_data - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 2 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
