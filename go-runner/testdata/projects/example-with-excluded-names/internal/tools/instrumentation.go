package tools

// Process processes data using target and node_modules utilities
func Process(data string) string {
	return processTarget(data) + processNodeModules(data)
}
