#!/usr/bin/env bash

function replace_in_go_files() {
    local search=$1
    local replace=$2
    find . -type f -name "*.go" -exec sed -i "s|$search|$replace|g" {} +
}

function fix_project() {
    rm -rf .git go.mod go.sum

    # Replace imports with patterns that only match in import statements
    # Match "testing" at start of line (after whitespace) to avoid string literals
    find . -type f -name "*.go" -exec sed -i 's/^[[:space:]]*"testing"$/testing "github.com\/CodSpeedHQ\/codspeed-go\/testing\/testing"/g' {} +
    find . -type f -name "*.go" -exec sed -i 's/^[[:space:]]*"testing")/testing "github.com\/CodSpeedHQ\/codspeed-go\/testing\/testing")/g' {} +
    find . -type f -name "*.go" -exec sed -i 's/^[[:space:]]*"testing",/testing "github.com\/CodSpeedHQ\/codspeed-go\/testing\/testing",/g' {} +

    # Handle single import statements: import "testing"
    find . -type f -name "*.go" -exec sed -i 's/^import "testing"$/import testing "github.com\/CodSpeedHQ\/codspeed-go\/testing\/testing"/g' {} +

    find . -type f -name "*.go" -exec sed -i 's/^[[:space:]]*"testing\/slogtest"$/slogtest "github.com\/CodSpeedHQ\/codspeed-go\/testing\/testing\/slogtest"/g' {} +
    find . -type f -name "*.go" -exec sed -i 's/^[[:space:]]*"testing\/slogtest")/slogtest "github.com\/CodSpeedHQ\/codspeed-go\/testing\/testing\/slogtest")/g' {} +
    find . -type f -name "*.go" -exec sed -i 's/^[[:space:]]*"testing\/slogtest",/slogtest "github.com\/CodSpeedHQ\/codspeed-go\/testing\/testing\/slogtest",/g' {} +

    go mod tidy
    go fmt ./...
}

git clone -b v0.3.4 https://github.com/thejerf/slogassert.git
pushd slogassert
replace_in_go_files '"github.com/thejerf/slogassert' '"github.com/CodSpeedHQ/codspeed-go/pkg/slogassert'
fix_project
popd

git clone -b v1.14.6 https://github.com/frankban/quicktest
pushd quicktest
replace_in_go_files '"github.com/frankban/quicktest' '"github.com/CodSpeedHQ/codspeed-go/pkg/quicktest'
fix_project
popd

git clone -b v1.4.3 https://github.com/go-logr/logr.git
pushd logr
replace_in_go_files '"github.com/go-logr/logr' '"github.com/CodSpeedHQ/codspeed-go/pkg/logr'
fix_project
popd

git clone -b v1.2.2 https://github.com/go-logr/stdr.git
pushd stdr
replace_in_go_files '"github.com/go-logr/stdr' '"github.com/CodSpeedHQ/codspeed-go/pkg/stdr'
# stdr imports logr, so we need to replace those imports too
replace_in_go_files '"github.com/go-logr/logr' '"github.com/CodSpeedHQ/codspeed-go/pkg/logr'
fix_project
popd

git clone -b v1.11.1 https://github.com/stretchr/testify.git
pushd testify
replace_in_go_files '"github.com/stretchr/testify' '"github.com/CodSpeedHQ/codspeed-go/pkg/testify'
fix_project
popd
