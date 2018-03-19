## 0.9.3
- Various UI/UX improvements in subcommands
  - Added --info-only flag to encode mode to show info about encoding
  - Added file and container sizes to encode mode stats
- Added calc mode to show detailed info about encoding configuration

## 0.9.2
- Made decode mode output file path determination more robust
  - Only the file part of the SNM field is used rather than the entire path when computing the final output path
- Added `--info-only` flag to encode mode
  - Using the flag shows various calculation results and statistical information

## 0.9.1
- Fixed encode mode output file determination logic
  - Prior to this version, encode mode would append the entire input path to the output path if output path is a directory, instead of just appending only the file name part

## 0.9.0
- Base version
