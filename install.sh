#!/bin/bash

# check if the argument is either --help or -h or none and then display usage
if [[ $# -eq 0 || $1 == "--help" || $1 == "-h" ]]; then
  echo "Usage: $0 DIRECTORY"
  exit 1
fi

# verify if the directory specified as the argument exists
if [ ! -d "$1" ]; then
    echo "Directory $1 does not exist."
    exit 1
fi

# proceed with the build
cargo build -r

SERVER_PATH=$PWD/target/release/server
cd $1
# Creates the directory if it doesn't exist, and does not throw an error if the directory already exists.
mkdir -p .bsp
cat > .bsp/cargo-bsp.json << EOF
{
  "name": "cargo-bsp",
  "argv": [
    "$SERVER_PATH"
  ],
  "version": "0.1.0",
  "bspVersion": "2.1.0",
  "languages": [
    "rust"
  ]
}
EOF

# print out the success message in green
echo -e "\033[0;32mInstallation succeeded!\033[0m"