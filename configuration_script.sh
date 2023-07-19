cargo build
echo Enter the path of the Rust project
read PROJECT_PATH
SERVER_PATH=$PWD
cd $PROJECT_PATH
mkdir .bsp
cat >> .bsp/cargo-bsp.json << EOF
{
  "name": "cargobsp",
  "argv": [
    "$SERVER_PATH/target/debug/server"
  ],
  "version": "0.1.0",
  "bspVersion": "2.1.0-M4",
  "languages": [
    "rust"
  ]
}