#!/usr/bin/env bash

function replace_in_go_files() {
    local search=$1
    local replace=$2
    find . -type f -name "*.go" -exec sed -i "s|$search|$replace|g" {} +
}

function fix_project() {
    rm -rf .git go.mod go.sum

    replace_in_go_files '"testing"' 'testing "github.com/CodSpeedHQ/codspeed-go/compat/testing"'
    replace_in_go_files '"testing/slogtest"' 'slogtest "github.com/CodSpeedHQ/codspeed-go/compat/testing/slogtest"'

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
