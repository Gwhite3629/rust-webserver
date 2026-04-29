
Simple rust webserver that attempts to conform to standards.

## Features

### Gzip Compression

All sent content bodies can be compressed before being chunked

### Chunked-transfer encoding

Rudimentary splitting of data into (small) chunks in order to demonstrate functionality that will be more useful with streamed connections

### Configurability

Very simple currently, define server root directory and host ip/port

### Stateless design

Attempt at REST conformity, though the ability to implement custom methods does not guarantee this

### Multithreading

Each request is handled by a seperate thread allowing for multiple connections and quick re-loading of resources

### HTTPS

How to make a key and identity file:

```cd cert```

Generate root certificate key:
```openssl genrsa -out root.pem 2048```

Generate root certificate self-signed certificate:
```openssl req -x509 -new -sha256 -key root.pem -config root.conf -extensions v3_ca -out root.crt```

Verify certificate content:
```openssl x509 -in root.crt -text -noout```

Generate intermediate certificate key:
```openssl genrsa -out intermediate.pem 2048```

Generate intermediate certificate csr:
```openssl req -new -sha256 -key intermediate.pem -config intermediate.conf -out intermediate.csr```

Generate signed intermediate certificate
```openssl x509 -days 365 -req -in intermediate.csr -CA root.crt -CAkey root.pem -CAcreateserial -out intermediate.crt```

Generate user key:
```openssl genrsa -out leaf.pem 2048```

Generate user csr:
```openssl req -new -sha256 -key leaf.pem -config leaf_req.conf -out leaf.csr```

Generate signed user certificate:
```openssl x509 -days 365 -req -in leaf.csr -CA intermediate.crt -CAkey intermediate.pem -CAcreateserial -out leaf.crt```

Combine certs into chain pfx file
```openssl pkcs12 -export -out identity.pfx -inkey leaf.pem -in leaf.crt -certfile intermediate.crt -certfile root.crt```

Code expects default as password, could make this a config value

Ensure that the server config file mentions the location of the identity.pfx file

Import the root.crt to the browser

## TODO

### Persistent / streamed connections

Make use of TCP and QUIC and allow connection multiplexing / keep-alive for HTTP 1.1

### More compression methods

brotli, deflate, maybe an interface for arbitrary stream writers

### SSL / TLS

HTTPS or less secure deprecated methods for old applications / testing

### All versions of HTTP

1.0, 1.1, 2.0, 3.0

### Other web/URI protocols

ftp, irc, ssh

### Dynamic web programming languages

PHP, JavaScript

### Proxy capabilities

Forward / Reverse abilities

### More modular method handling

Meta-programming config format that allows for definition of state-machines or pattern matching a series of header values

### Stress testing / benchmarking

Add some way to create synthetic loads and test many connections. Additionally add unit tests to every module