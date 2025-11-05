// Licensed under the MIT license, see LICENSE file for details.

//go:build !go1.17
// +build !go1.17

package quicktest_test

import (
	testing "github.com/CodSpeedHQ/codspeed-go/testing/testing"
	"os"

	qt "github.com/CodSpeedHQ/codspeed-go/pkg/quicktest"
)

const envName = "SOME_VAR"

func TestCSetenv(t *testing.T) {
	c := qt.New(t)
	os.Setenv(envName, "initial")
	testCleanup(t, func(c *qt.C) {
		c.Setenv(envName, "new value")
		c.Check(os.Getenv(envName), qt.Equals, "new value")
	})
	c.Check(os.Getenv(envName), qt.Equals, "initial")
}

func TestCSetenvWithUnsetVariable(t *testing.T) {
	c := qt.New(t)
	os.Unsetenv(envName)
	testCleanup(t, func(c *qt.C) {
		c.Setenv(envName, "new value")
		c.Check(os.Getenv(envName), qt.Equals, "new value")
	})
	_, ok := os.LookupEnv(envName)
	c.Assert(ok, qt.IsFalse)
}
