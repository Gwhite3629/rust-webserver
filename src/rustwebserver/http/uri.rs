
use core::fmt::Display;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
        static ref URI_REGEX: Regex = Regex::new(
            r"^((?<scheme>[^:/?#]+):)?(//(?<authority>[^/?#]*))?(?<path>[^?#]*)(\?(?<query>[^#]*))?(#(?<fragment>.*))?$"
        ).unwrap();
    } 

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

impl Scheme {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Scheme { scheme: String::new() }
        } else {
            Scheme { scheme: s.clone() }
        }
    }
}

// authority   = [ userinfo "@" ] host [ ":" port ] (RFC 3986)
// userinfo    = *( unreserved / pct-encoded / sub-delims / ":" )
// host        = IP-literal / IPv4address / reg-name 
// port        = *DIGIT

#[derive(Debug)]
struct Authority {
    userinfo: String,
    host: String,
    port: u16,
}

impl Authority {
    pub fn new(s: &String) -> Self {

        if s.is_empty() {
            Authority {userinfo: String::new(),host: String::new(),port: 0}
        } else {


            let uinfore: Regex = Regex::new(r"^((?<userinfo>[^/?#@]*)@)?").unwrap();
            let portre: Regex = Regex::new(r"[^?#/@:](:(?<port>[0-9]+))$").unwrap();

            let mut infostring = match uinfore.captures(s) {
                Some(res) => match res.name("userinfo") {
                    Some(string) => string.as_str().to_string(),
                    None => String::new(),
                },
                None => String::new(),
            };

            let mod_s = if !infostring.is_empty() {
                infostring.push('@');
                s.replace(infostring.as_str(),"")
            } else {
                s.clone()
            };

            let mut portstring = match portre.captures(&mod_s) {
                Some(res) => match res.name("port") {
                    Some(string) => string.as_str().to_string(),
                    None => String::new(),
                },
                None => String::new(),
            };

            let final_s = if !portstring.is_empty() {
                portstring.insert(0, ':');
                mod_s.replace(portstring.as_str(), "")
            } else {
                mod_s.clone()
            };

            Authority { userinfo: infostring, host: final_s.clone(), port: portstring.parse::<u16>().unwrap() }
        }
    } 
}

// host        = IP-literal / IPv4address / reg-name 
// 
// IP-literal = "[" ( IPv6address / IPvFuture  ) "]"
//  ?<IP-literal>(
//      \[
//          ?<IPv6address>()
//          ?<IPvFuture>()
//      \])
//  ?<Ipv4address>()
//  ?<reg-name>()
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

impl Path {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Path { path: String::new() }
        } else {
            Path { path: s.clone() }
        }
    }
}

// query       = *( pchar / "/" / "?" )
#[derive(Default, Debug)]
struct Query {
    query: String,
}

impl Query {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Query { query: String::new() }
        } else {
            Query { query: s.clone() }
        }
    }
}

// fragment    = *( pchar / "/" / "?" )
#[derive(Default, Debug)]
struct Fragment {
    fragment: String,
}

impl Fragment {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Fragment { fragment: String::new() }
        } else {
            Fragment { fragment: s.clone() }
        }
    }
}

macro_rules! urimatch {
    ($s:expr, $m:expr, $c:expr) => {
        {
        let s = match $c {
            Some(res) =>  match res.name($m) {
                Some(string) => string.as_str().to_string(),
                None => String::new(),
            },
            None => String::new(),
        };
        s
        }
    };
}

impl URI {
    pub fn new(s: &String) -> Self {
        let cap = URI_REGEX.captures(s);

        let schemestring = urimatch!(s, "scheme", &cap);
        let authstring = urimatch!(s, "authorization", &cap);
        let pathstring = urimatch!(s, "path", &cap);
        let querystring = urimatch!(s, "query", &cap);
        let fragmentstring = urimatch!(s, "fragment", &cap);        

        return URI { 
            scheme: Scheme::new(&schemestring),
            authority: Authority::new(&authstring),
            path: Path::new(&pathstring),
            query: Query::new(&querystring),
            fragment: Fragment::new(&fragmentstring),
        }
    }
}

impl Display for URI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!{f,
            "\tScheme: {}\n
\tAuthority:\n
\t\tUserInfo: {}\n
\t\tHost:Port: {}:{}\n
\tPath: {}\n
\tQuery: {}\n
\tFragment: {}\n",
            self.scheme.scheme,
            self.authority.userinfo,
            self.authority.host,
            self.authority.port.to_string(),
            self.path.path,
            self.query.query,
            self.fragment.fragment,
        }
    }
}