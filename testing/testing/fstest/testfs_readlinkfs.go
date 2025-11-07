//go:build go1.25
// +build go1.25

package fstest

import "io/fs"

func (t *fsTester) checkReadLinkFS(path string, einfo fs.FileInfo, fentry string) {
	if fsys, ok := t.fsys.(fs.ReadLinkFS); ok {
		info2, err := fsys.Lstat(path)
		if err != nil {
			t.errorf("%s: fsys.Lstat: %v", path, err)
			return
		}
		fientry2 := formatInfoEntry(info2)
		if fentry != fientry2 {
			t.errorf("%s: mismatch:\n\tentry = %s\n\tfsys.Lstat(...) = %s", path, fentry, fientry2)
		}
		feinfo := formatInfo(einfo)
		finfo2 := formatInfo(info2)
		if feinfo != finfo2 {
			t.errorf("%s: mismatch:\n\tentry.Info() = %s\n\tfsys.Lstat(...) = %s\n", path, feinfo, finfo2)
		}
	}
}
