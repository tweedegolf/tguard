package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"

	irma "github.com/privacybydesign/irmago"
)

type Configuration struct {
	irmaconf *irma.Configuration
}

func (c *Configuration) verify_signature(w http.ResponseWriter, r *http.Request) {
	var message irma.SignedMessage
	body, err := io.ReadAll(r.Body)
	if err != nil {
		fmt.Println(err)
		w.WriteHeader(500)
		return
	}
	err = json.Unmarshal(body, &message)
	if err != nil {
		w.WriteHeader(400)
		return
	}
	attributes, status, err := message.Verify(c.irmaconf, nil)
	if err != nil || status != irma.ProofStatusValid {
		w.WriteHeader(400)
		return
	}
	mapped_attributes := make(map[irma.AttributeTypeIdentifier]string)
	for _, al := range attributes {
		for _, attr := range al {
			if attr.RawValue != nil {
				mapped_attributes[attr.Identifier] = *attr.RawValue
			}
		}
	}
	json_attributes, err := json.Marshal(&mapped_attributes)
	if err != nil {
		fmt.Println(err)
		w.WriteHeader(500)
		return
	}
	_, err = w.Write(json_attributes)
	if err != nil {
		fmt.Println(err)
	}
}

func main() {
	scheme_dir := os.Getenv("SCHEMES_DIR")
	irmaconf, err := irma.NewConfiguration(scheme_dir, irma.ConfigurationOptions{})
	if err != nil {
		panic(err)
	}
	irmaconf.ParseFolder()
	if len(irmaconf.SchemeManagers) == 0 {
		irmaconf.DownloadDefaultSchemes()
	}
	irmaconf.AutoUpdateSchemes(15)
	conf := &Configuration{
		irmaconf,
	}
	http.HandleFunc("/api/verify", conf.verify_signature)
	panic(http.ListenAndServe(":8080", nil))
}
