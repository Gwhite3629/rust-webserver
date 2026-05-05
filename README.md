
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
```openssl genrsa -out root_key.pem 2048```

Generate root certificate self-signed certificate:
```openssl req -x509 -new -sha256 -key root_key.pem -config root.conf -extensions v3_ca -out root_cert.pem```

Verify certificate content:
```openssl x509 -in root_cert.pem -text -noout```

Generate intermediate certificate key:
```openssl genrsa -out intermediate_key.pem 2048```

Generate intermediate certificate csr:
```openssl req -new -sha256 -key intermediate_key.pem -config intermediate.conf -out intermediate.csr```

Generate signed intermediate certificate
```openssl x509 -days 365 -req -in intermediate.csr -CA root_cert.pem -CAkey root_key.pem -CAcreateserial -out intermediate_cert.pem```

Generate user key:
```openssl genrsa -out leaf_key.pem 2048```

Generate user csr:
```openssl req -new -sha256 -key leaf_key.pem -config leaf_req.conf -out leaf.csr```

Generate signed user certificate:
```openssl x509 -days 365 -req -in leaf.csr -CA intermediate_cert.pem -CAkey intermediate_key.pem -CAcreateserial -out leaf_cert.pem```

Combine certs into chain pfx file
```cat leaf_cert.pem intermediate_cert.pem root_cert.pem > Identity.pem```

Ensure that the server config file mentions the location of the Identity.pem and leaf_key.key

Import the root_cert.pem to the browser

## TODO

### Persistent / streamed connections

Make use of TCP and QUIC and allow connection multiplexing / keep-alive for HTTP 1.1

### More compression methods

brotli, deflate, maybe an interface for arbitrary stream writers

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