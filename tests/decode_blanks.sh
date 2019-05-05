#!/bin/bash

exit_code=0

corrupt() {
  dd if=/dev/zero of=$2 bs=10 count=1 seek=$1 conv=notrunc &>/dev/null
}

echo "Testing version 1"
echo "Encoding"
output=$(./../blkar encode --sbx-version 1 --json -f dummy)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo "Collecting base statistics"
output=$(./../blkar decode --json -f dummy.sbx)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
blocks_processed=$(echo $output | jq -r ".stats.numberOfBlocksProcessed")
okay_meta=$(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata")
okay_data=$(echo $output | jq -r ".stats.numberOfBlocksDecodedData")
failed_blocks=$(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode")

echo "Corrupting container"
corrupt 512 dummy.sbx

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == $[okay_data - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == $[failed_blocks+1] ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Comparing with anticipated output"
cp dummy data_chunk_orig
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=0 conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo "Encoding"
output=$(./../blkar encode --sbx-version 1 --json -f dummy)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo "Corrupting container"
corrupt 4096 dummy.sbx

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == $[okay_data - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == $[failed_blocks+1] ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Comparing with anticipated output"
cp dummy data_chunk_orig
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=$[7 * 496] conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo "Encoding"
output=$(./../blkar encode --sbx-version 1 --json -f dummy)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo "Corrupting container"
corrupt 4096  dummy.sbx
corrupt 5632  dummy.sbx
corrupt 13824 dummy.sbx

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.sbx data_chunk)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == 1 ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == $[okay_data - 3] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == $[failed_blocks+3] ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Comparing with anticipated output"
cp dummy data_chunk_orig
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=$[7 * 496] conv=notrunc 2>/dev/null
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=$[10 * 496] conv=notrunc 2>/dev/null
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=$[26 * 496] conv=notrunc 2>/dev/null
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
output=$(./../blkar encode --json -f dummy --sbx-version 17 --rs-data 3 --rs-parity 2)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo "Collecting base statistics"
output=$(./../blkar decode --json -f dummy.ecsbx)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
blocks_processed=$(echo $output | jq -r ".stats.numberOfBlocksProcessed")
okay_meta=$(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata")
okay_data=$(echo $output | jq -r ".stats.numberOfBlocksDecodedData")
okay_parity=$(echo $output | jq -r ".stats.numberOfBlocksDecodedParity")
failed_blocks=$(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode")

echo "Corrupting container"
corrupt 512 dummy.ecsbx

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.ecsbx data_chunk)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == $[okay_meta - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == $okay_data ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == $okay_parity ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == $[failed_blocks+1] ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Comparing with anticipated output"
cp dummy data_chunk_orig
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo "Encoding"
output=$(./../blkar encode --json -f dummy --sbx-version 17 --rs-data 3 --rs-parity 2)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo "Corrupting container"
corrupt 4096 dummy.ecsbx

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.ecsbx data_chunk)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == $okay_meta ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == $[okay_data - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == $okay_parity ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == $[failed_blocks+1] ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Comparing with anticipated output"
cp dummy data_chunk_orig
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=$[3 * 496] conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo "Encoding"
output=$(./../blkar encode --json -f dummy --sbx-version 17 --rs-data 3 --rs-parity 2)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo "Corrupting container"
corrupt 5632  dummy.ecsbx

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.ecsbx data_chunk)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == $okay_meta ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == $okay_data ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == $[okay_parity - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == $[failed_blocks+1] ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Comparing with anticipated output"
cp dummy data_chunk_orig
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo "Encoding"
output=$(./../blkar encode --json -f dummy --sbx-version 17 --rs-data 3 --rs-parity 2)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo "Corrupting container"
corrupt 4096  dummy.ecsbx
corrupt 5632  dummy.ecsbx
corrupt 12800 dummy.ecsbx

echo -n "Decoding"
output=$(./../blkar decode --json -f dummy.ecsbx data_chunk)
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
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedMetadata") == $okay_meta ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedData") == $[okay_data - 2] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksDecodedParity") == $[okay_parity - 1] ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedToDecode") == $[failed_blocks+3] ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Comparing with anticipated output"
cp dummy data_chunk_orig
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=$[3 * 496] conv=notrunc 2>/dev/null
dd if=/dev/zero of=data_chunk_orig bs=1 count=496 seek=$[14 * 496] conv=notrunc 2>/dev/null
cmp data_chunk data_chunk_orig
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
