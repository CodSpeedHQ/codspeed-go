# codspeed-go

[![CI](https://github.com/AvalancheHQ/codspeed-go/actions/workflows/ci.yml/badge.svg)](https://github.com/AvalancheHQ/codspeed-go/actions/workflows/ci.yml)
[![Go Report Card](https://goreportcard.com/badge/github.com/AvalancheHQ/codspeed-go)](https://goreportcard.com/report/github.com/AvalancheHQ/codspeed-go)
[![codecov](https://codecov.io/gh/AvalancheHQ/codspeed-go/branch/main/graph/badge.svg)](https://codecov.io/gh/AvalancheHQ/codspeed-go)

A Go CLI tool for running Go benchmarks and generating CodSpeed-compatible performance reports with advanced statistical analysis.

## Prerequisites

- Go 1.21 or later
- Nix (optional, for reproducible environment)

## Installation

### From Source

```bash
git clone https://github.com/AvalancheHQ/codspeed-go.git
cd codspeed-go
go build -o codspeed-go .
```

### Using Nix

```bash
# Development environment
nix develop

# With linting tools
nix develop .#lint

# CI environment
nix develop .#ci
```

## Usage

### Basic Commands

```bash
# Show help
./codspeed-go

# Run all benchmarks in current directory
./codspeed-go run

# Run benchmarks with pattern filtering
./codspeed-go run BenchmarkFibonacci20

# Generate CodSpeed-compatible JSON report
./codspeed-go analyze

# Analyze specific benchmark pattern
./codspeed-go analyze BenchmarkFibonacci20
```

## Development

### Development Environment

```bash
# Enter development shell
nix develop

# Install pre-commit hooks
nix develop .#lint

# Run tests
go test -v ./...

# Run linter
golangci-lint run

# Build
go build -o codspeed-go .
```
