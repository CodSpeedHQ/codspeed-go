package tools

// processTarget is called from target.go
func processTarget(data string) string {
	return "target:" + data
}

// processNodeModules is called from node_modules.go
func processNodeModules(data string) string {
	return "|nm:" + data
}
