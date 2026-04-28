
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

Generate a key using:

Code currently expects "default" as password

```
openssl req -x509 -out localhost.crt -keyout localhost.key \
  -newkey rsa:2048 -nodes -sha256 \
  -subj '/CN=localhost' -extensions EXT -config <( \
   printf "[dn]\nCN=localhost\n[req]\ndistinguished_name = dn\n[EXT]\nsubjectAltName=DNS:localhost\nkeyUsage=digitalSignature\nextendedKeyUsage=serverAuth")

openssl pkcs12 -export -out identity.pfx \
 -inkey localhost.key \
 -in localhost.crt 

```

With this add an alias to /etc/hosts to indicate 127.0.0.1 is localhost
Then add the crt file to the browsers certificates


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