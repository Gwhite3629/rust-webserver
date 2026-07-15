#!/bin/bash
openssl genrsa -out root_key.pem 2048
openssl req -x509 -new -sha256 -key root_key.pem -config root.conf -extensions v3_ca -out root_cert.pem
openssl x509 -in root_cert.pem -text -noout
openssl genrsa -out intermediate_key.pem 2048
openssl req -x509 -new -sha256 -key intermediate_key.pem -config intermediate.conf -extensions v3_ca -out intermediate.csr
openssl x509 -days 365 -in intermediate.csr -CA root_cert.pem -CAkey root_key.pem -out intermediate_cert.pem
openssl x509 -in intermediate_cert.pem -text -noout
openssl genrsa -out leaf_key.pem 2048
openssl req -x509 -new -sha256 -key leaf_key.pem -config leaf.conf -extensions v3_ca -out leaf.csr
openssl x509 -days 365 -in leaf.csr -CA intermediate_cert.pem -CAkey intermediate_key.pem -CAcreateserial -out leaf_cert.pem
openssl x509 -in leaf_cert.pem -text -noout
cat leaf_cert.pem intermediate_cert.pem root_cert.pem > Identity.pem