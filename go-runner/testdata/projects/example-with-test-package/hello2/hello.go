package hello2

import "fmt"

func Say(message string) string {
	if message == "" {
		return ""
	}

	return makeMessage(message)
}

func makeMessage(message string) string {
	return fmt.Sprintf("hello %s", message)
}
