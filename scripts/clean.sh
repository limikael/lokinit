#!/bin/bash

MoveAfter=.
if [ -d "scripts" ]
then
    MoveAfter=..
    cd scripts
fi

rm -rf build/*
rm -rf ../target/aarch64-apple-ios/release/*

cd $MoveAfter