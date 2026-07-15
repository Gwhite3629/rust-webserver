
Simple rust webserver that attempts to conform to standards.

## Features

### Gzip Compression

All sent content bodies can be compressed before being chunked

### Chunked-transfer encoding

Rudimentary splitting of data into (small) chunks in order to demonstrate functionality that will be more useful with streamed connections

### Configurability

Very simple currently, define server root directory and host ip/port

### Static content serving

Provides HEAD, GET and TRACE methods

### Stateless design

Attempt at REST conformity, though the ability to implement custom methods does not guarantee this

### Async requests

Leveraging the mio library for faster response time

### Virtual hosting

Allows multiple domains to be hosted and routed based on internal IP and ports

### URL Redirection and authorization

Config allows URL resources to be aliased and protected as realms with basic or digest authentication. Nested authentication is undefined behavior but should short circuit to the lowest (closest to root) level. Individual resources can be assigned auth with the same or different realm. This may change to allow definition of realm auth and then assigning a resource a realm.

### HTTPS

How to make a key and identity file:

```cd cert```

```run create.sh```

Ensure that the server config file mentions the location of the Identity.pem and leaf_key.key

The localhost and 127.0.0.1 SANs are specified in the leaf.conf file and can be changed or added to

Import the root_cert.pem to the browser

## TODO

### Common features of other browsers

CGI, fastCGI, SSI, logging, caching, throttling / rate limiting

### Persistent / streamed connections

Make use of TCP and QUIC and allow connection multiplexing / keep-alive for HTTP 1.1

### TLS 1.3 and advanced features

SNI, HTTP strict transport security

### More compression methods

brotli, deflate, maybe an interface for arbitrary stream writers

### All versions of HTTP

1.1, 2.0, 3.0

### Other web/URI protocols

ftp, irc, ssh, ftps

### Dynamic web programming languages

PHP, JavaScript

### Proxy capabilities

Forward / Reverse abilities

### Stress testing / benchmarking

Add some way to create synthetic loads and test many connections. Additionally add unit tests to every module