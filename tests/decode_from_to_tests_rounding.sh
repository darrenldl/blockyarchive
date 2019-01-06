#!/bin/bash

exit_code=0

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

offset=$[1 + RANDOM % 100]

file_size=$(ls -l dummy | awk '{ print $5 }')

echo "Testing version 1"
echo "Encoding"
output=$(./../blkar encode --json -f dummy)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $offset --to-exc 512)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $offset --to-exc 1024)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
dd if=dummy     of=data_chunk_orig bs=1 count=496        skip=0 seek=0 conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $[offset + 1024] --to-inc 1024)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
dd if=dummy     of=data_chunk_orig bs=1 count=496        skip=496 seek=496 conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $[offset + 6144] --to-exc 69120)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 123 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
dd if=dummy     of=data_chunk_orig bs=1 count=$[496 * 123]        skip=$[11 * 496] seek=$[11 * 496] conv=notrunc 2>/dev/null
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

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $offset --to-exc 512)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $offset --to-exc 2048)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 3 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
dd if=dummy     of=data_chunk_orig bs=1 count=496        skip=0 seek=0 conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $[offset + 2048] --to-inc 2048)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
dd if=dummy     of=data_chunk_orig bs=1 count=496        skip=496 seek=496 conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk --from $[offset + 37376] --to-exc 112640)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 0 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == 105 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == 42 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking if output data chunk matches the original file portion"
dd if=/dev/zero of=data_chunk_orig bs=1 count=$file_size skip=0 2>/dev/null
dd if=dummy     of=data_chunk_orig bs=1 count=$[496 * 105]        skip=$[50 * 496] seek=$[50 * 496] conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
