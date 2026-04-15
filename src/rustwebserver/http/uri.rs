
use core::fmt::Display;
use std::{net::Ipv4Addr, str::FromStr};


// URI defined by RFC3986


// URI         = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
//
//      hier-part   = "//" authority path-abempty
//                  / path-absolute
//                  / path-rootless
//                  / path-empty

#[derive(Debug)]
pub struct URI {
    scheme: Scheme,
    authority: Authority,
    path: Path,
    query: Query,
    fragment: Fragment,
}

// scheme      = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ) (RFC 3986)
#[derive(Clone, Debug)]
struct Scheme {
    scheme: String,
}

// authority   = [ userinfo "@" ] host [ ":" port ] (RFC 3986)
// userinfo    = *( unreserved / pct-encoded / sub-delims / ":" )
// host        = IP-literal / IPv4address / reg-name 
// port        = *DIGIT

#[derive(Debug)]
struct Authority {
    userinfo: String,
    host: Host,
    port: u8,
}

// host        = IP-literal / IPv4address / reg-name 
//
// IP-literal = "[" ( IPv6address / IPvFuture  ) "]"
//
//      IPvFuture  = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
//
// IPv6address =                            6( h16 ":" ) ls32
//                  /                       "::" 5( h16 ":" ) ls32
//                  / [               h16 ] "::" 4( h16 ":" ) ls32
//                  / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) ls32
//                  / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) ls32
//                  / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   ls32
//                  / [ *4( h16 ":" ) h16 ] "::"              ls32
//                  / [ *5( h16 ":" ) h16 ] "::"              h16
//                  / [ *6( h16 ":" ) h16 ] "::"
//
//      ls32        = ( h16 ":" h16 ) / IPv4address
//                  ; least-significant 32 bits of address
//
//      h16         = 1*4HEXDIG
//                  ; 16 bits of address represented in hexadecimal
//
//      IPv4address = dec-octet "." dec-octet "." dec-octet "." dec-octet
//
//      dec-octet   = DIGIT                 ; 0-9
//                  / %x31-39 DIGIT         ; 10-99
//                  / "1" 2DIGIT            ; 100-199
//                  / "2" %x30-34 DIGIT     ; 200-249
//                  / "25" %x30-35          ; 250-255
//
// reg-name    = *( unreserved / pct-encoded / sub-delims )
#[derive(Debug)]
struct Host {
    ipliteral: String,
    ipv4address: Ipv4Addr,
    regname: String,
}

// path          = path-abempty    ; begins with "/" or is empty
//                    / path-absolute   ; begins with "/" but not "//"
//                    / path-noscheme   ; begins with a non-colon segment
//                    / path-rootless   ; begins with a segment
//                    / path-empty      ; zero characters
//
//      path-abempty  = *( "/" segment )
//      path-absolute = "/" [ segment-nz *( "/" segment ) ]
//      path-noscheme = segment-nz-nc *( "/" segment )
//      path-rootless = segment-nz *( "/" segment )
//      path-empty    = 0<pchar>
//
//      segment       = *pchar
//      segment-nz    = 1*pchar
//      segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )
//                    ; non-zero-length segment without any colon ":"
//
//      pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"

#[derive(Debug)]
struct Path {
    path: String,
}

// query       = *( pchar / "/" / "?" )
#[derive(Default, Debug)]
struct Query {
    query: String,
}

// fragment    = *( pchar / "/" / "?" )
#[derive(Default, Debug)]
struct Fragment {
    fragment: String,
}

impl URI {
    pub fn new(s: Vec<String>, h: Vec<String>) -> Self {
        return URI { 
            scheme: Scheme { 
                scheme: s[2].clone(),
            },
            authority: Authority {
                userinfo: String::default(),
                host: Host {
                    ipliteral: String::default(), 
                    ipv4address: Ipv4Addr::from_str(&h[1]).unwrap(), 
                    regname: String::default(),
                },
                port: h[2].parse::<u8>().unwrap(),
            }, 
            path: Path {
                    path: s[1].clone(),
            },
            query: Query::default(),
            fragment: Fragment::default(),
        }
    }
}

impl Display for URI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!{f,
            "Scheme: {}\n
            Authority:\n
            \tUserInfo: {}\n
            \tHost:Port: {}:{}\n
            Path: {}\n
            Query: {}\n
            Fragment: {}\n",
            self.scheme.scheme,
            self.authority.userinfo,
            self.authority.host.ipv4address.to_string(),
            self.authority.port.to_string(),
            self.path.path,
            self.query.query,
            self.fragment.fragment,
        }
    }
}