cargo build -r
SERVER_PATH=$PWD/target/release/server
cd $1
mkdir .bsp
cat >> .bsp/cargo-bsp.json << EOF
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
