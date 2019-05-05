#!/bin/bash

exit_code=0

offset=$[1 + RANDOM % 100]

source functions.sh

file_size=$(ls -l dummy | awk '{ print $5 }')

echo "Testing version 1"
echo "Encoding"
output=$(./../blkar encode --sbx-version 1 --json -f dummy)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

mv dummy.sbx dummy.sbx.tmp
touch dummy.sbx
truncate -s $offset dummy.sbx
cat dummy.sbx.tmp >> dummy.sbx
rm dummy.sbx.tmp

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.sbx data_chunk --from $offset --to-exc 512 --force-misalign)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=dummy.sbx of=data_chunk_orig bs=1 count=512 skip=$offset 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.sbx data_chunk --from $offset --to-exc 1024 --force-misalign)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=dummy.sbx of=data_chunk_orig bs=1 count=$[2*512] skip=$offset 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.sbx data_chunk --from $[offset + 1024] --to-inc 1024 --force-misalign)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=/dev/zero     of=data_chunk_orig bs=1 count=$[1024 + 512] skip=0 2>/dev/null
dd if=dummy.sbx     of=data_chunk_orig bs=1 count=512        skip=$[offset + 1024] seek=1024 conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.sbx data_chunk --from $[offset + 6144] --to-exc 69120 --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 123 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 123 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 123 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$[(11 + 123) * 512] skip=0 2>/dev/null
dd if=dummy.sbx of=data_chunk_orig bs=1 count=$[512 * 123]        skip=$[offset + 12 * 512] seek=$[12 * 512] conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo ""

echo "Testing version 17"
echo "Encoding"
output=$(./../blkar encode --json -f dummy --sbx-version 17 --rs-data 5 --rs-parity 2)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

mv dummy.ecsbx dummy.ecsbx.tmp
touch dummy.ecsbx
truncate -s $offset dummy.ecsbx
cat dummy.ecsbx.tmp >> dummy.ecsbx
rm dummy.ecsbx.tmp

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.ecsbx data_chunk --from $offset --to-exc 512 --force-misalign)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 3 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=dummy.ecsbx of=data_chunk_orig bs=1 count=$[3*512] skip=$offset 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.ecsbx data_chunk --from $offset --to-exc 2048 --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 4 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 3 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 3 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=dummy.ecsbx of=data_chunk_orig bs=1 count=2048       skip=$offset seek=0 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.ecsbx data_chunk --from $[offset + 2048] --to-inc 2048 --force-misalign)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$[5*512] skip=0 2>/dev/null
dd if=dummy.ecsbx of=data_chunk_orig bs=1 count=512        skip=$[offset + 4*512] seek=$[4*512] conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Sorting"
output=$(./../blkar sort --json -f dummy.ecsbx data_chunk --from $[offset + 37376] --to-exc 112640 --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksProcessed") == 147 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedData") == 105 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksSortedParity") == 42 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData") == 105 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInSameOrderParity") == 42 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToSort") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking output blocks"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$[220 * 512] skip=0 2>/dev/null
dd if=dummy.ecsbx of=data_chunk_orig bs=1 count=$[147 * 512]        skip=$[offset + 73 * 512] seek=$[73 * 512] conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
