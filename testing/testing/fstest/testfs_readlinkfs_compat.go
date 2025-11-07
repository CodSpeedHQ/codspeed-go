//go:build !go1.25
// +build !go1.25

package fstest

import "io/fs"

// No-op on Go versions before 1.25 since fs.ReadLinkFS doesn't exist.
func (t *fsTester) checkReadLinkFS(path string, einfo fs.FileInfo, fentry string) {}
