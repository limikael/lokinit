#!/bin/bash

clear

# We need to be in the parent directory, eg where the rust project is, for cargo
# We'll move to MoveAfter when the script finishes
MoveAfter=.
if [ ! -d "scripts" ]
then
    cd ..
    MoveAfter=scripts
fi

# Verify the app name
AppName=$1
BinaryName=$2
BuildFolder="scripts/build/$AppName.app"
if [ ! -d $BuildFolder ]
then
    echo "ERROR: Invalid application to generate."
    echo "Format: ./mkipa.sh <ApplicationName> <BinaryToCompile>"
    exit 1
fi
if [ ! -f examples/$BinaryName.rs ]
then
    echo "ERROR: Invalid binary to compile."
    echo "Format: ./mkipa.sh <ApplicationName> <BinaryToCompile>"
    exit 1
fi
AppNameLower=$(echo "$AppName" | tr '[:upper:]' '[:lower:]')

# Compile the binary
echo "Compiling the app..."
cargo build --target aarch64-apple-ios --example $BinaryName

# Convert the .app into a .ipa
echo "Converting the app to an ipa..."
cd scripts/build
mkdir Payload
cp -r $AppName.app Payload/
cp ../../target/aarch64-apple-ios/release/examples/$BinaryName Payload/$AppName.app/$AppNameLower
rm $AppName.ipa
zip -r $AppName.ipa Payload
rm -rf Payload
cd ../..

# Finish
echo "Done."
cd $MoveAfter
