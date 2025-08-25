#!/usr/bin/env bash

git clone -b release-branch.go1.24 --depth 1 https://github.com/golang/go/

rm -rf internal testing

# We need to copy the testing/ package:
cp -r go/src/testing testing/

mkdir -p internal/cpu
mkdir -p internal/fuzz
mkdir -p internal/goarch
mkdir -p internal/race
mkdir -p internal/sysinfo
mkdir -p internal/testlog
mkdir -p internal/goexperiment

cp -r go/src/internal/cpu/* internal/cpu/
cp -r go/src/internal/fuzz/* internal/fuzz/
cp -r go/src/internal/goarch/* internal/goarch/
cp -r go/src/internal/race/* internal/race/
cp -r go/src/internal/sysinfo/* internal/sysinfo/
cp -r go/src/internal/testlog/* internal/testlog/
cp -r go/src/internal/goexperiment/* internal/goexperiment/

# Replace all `"internal/*"` imports with 'github.com/CodSpeedHQ/codspeed-go/testing/internal/'
find . -type f -name "*.go" -exec sed -i 's|"internal/|"github.com/CodSpeedHQ/codspeed-go/testing/internal/|g' {} +

# Apply the race package patch to remove abi dependency
patch -p1 --forward --reject-file=- < internal_race.patch || echo "Patch may have already been applied or needs manual intervention"
