#!/bin/bash

clear

# If we aren't in the scripts directory, move to it
# MoveAfter will get moved to when the script finishes
MoveAfter=.
if [ -d "scripts" ]
then
    cd scripts
    MoveAfter=..
fi

# Make build directories, if they don't exist
if [ ! -d "build" ]
then
    mkdir build
fi

# Read the app name
echo "What do you want to name the iOS app?"
read AppName
BuildFolder="build/$AppName.app"
AppNameLower=$(echo "$AppName" | tr '[:upper:]' '[:lower:]')

# Generate the application
echo "Creating '$AppName.app'..."
mkdir $BuildFolder
echo "|- Generating 'Info.plist'..."
cat > $BuildFolder/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
<key>CFBundleExecutable</key>
<string>$AppNameLower</string>
<key>CFBundleIdentifier</key>
<string>com.$AppNameLower</string>
<key>CFBundleName</key>
<string>$AppName</string>
<key>CFBundleVersion</key>
<string>1</string>
<key>CFBundleShortVersionString</key>
<string>1.0</string>
</dict>
</plist>
EOF

# Add a target for iOS
echo "Adding Rust target for iOS..."
rustup target add aarch64-apple-ios

# Finish
echo "Done."
cd $MoveAfter
