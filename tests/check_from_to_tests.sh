#!/bin/bash

exit_code=0

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

echo "Creating empty files"
touch dummy_empty1
touch dummy_empty2

echo -n "Encoding 1st file"
output=$(./../blkar encode --json -f dummy_empty1 --uid DEADBEEF0001)
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

echo -n "Encoding 2nd file"
output=$(./../blkar encode --json -f dummy_empty2 --uid DEADBEEF0002)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.fileUID") == "DEADBEEF0002" ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

echo "Crafting dummy disk file"
rm dummy_empty_disk &>/dev/null
cat dummy_empty1.sbx >> dummy_empty_disk
cat dummy_empty2.sbx >> dummy_empty_disk

echo "Checking dummy disk"

echo -n "Checking that blkar only checks the first block"
output=$(./../blkar rescue --json dummy_empty_disk rescued_data --from 0 --to-inc 511)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

corrupt 0 dummy_empty_disk

output=$(./../blkar rescue --json dummy_empty_disk rescued_data --from 0 --to-inc 511)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking that blkar only checks the second block"
output=$(./../blkar rescue --json dummy_empty_disk rescued_data --from 512 --to-inc 512)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

corrupt 0 dummy_empty_disk

output=$(./../blkar rescue --json dummy_empty_disk rescued_data --from 0 --to-inc 511)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 1 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking that blkar checks both blocks"
output=$(./../blkar rescue --json dummy_empty_disk rescued_data)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 2 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckMetadata") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksPassedCheckData") == 0 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 2 ]]; then
  echo " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
