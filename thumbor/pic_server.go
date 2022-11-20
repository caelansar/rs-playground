package main

import (
	"log"
	"net/http"
)

func main() {
	file := http.FileServer(http.Dir("/Users/caelansar/Pictures"))
	http.Handle("/static/", http.StripPrefix("/static/", file))
	err := http.ListenAndServe(":5000", nil)
	if err != nil {
		log.Println(err)
	}
}
