#!/bin/bash

# Ensure you're in the root directory of the project
cd "$(dirname "$0")/.."

# Run the build script for proto_definitions to generate the Protobuf files
cd proto_definitions
cargo build --release

# Copy the generated files to the core and admin_shell directories
cp -r src/generated ../core/src/generated
cp -r src/generated ../admin_shell/src/generated

echo "Protobuf files generated and copied to core and admin_shell."

