use core::fmt::Display;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
        static ref URI_REGEX: Regex = Regex::new(
            r"^((?<scheme>[^:/?#]+)://)?((?<authority>[^/?#]*))?(?<path>[^?#]*)(\?(?<query>[^#]*))?(#(?<fragment>.*))?$"
        ).unwrap();
    } 

// URI defined by RFC3986


// URI         = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
//
//      hier-part   = "//" authority path-abempty
//                  / path-absolute
//                  / path-rootless
//                  / path-empty

#[derive(Debug, Clone)]
pub struct URI {
    pub scheme: Scheme,
    pub authority: Authority,
    pub path: Path,
    pub query: Query,
    pub fragment: Fragment,
}

// scheme      = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ) (RFC 3986)
#[derive(Debug, PartialEq, Clone)]
pub struct Scheme (String);

impl Scheme {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Scheme(String::new())
        } else {
            Scheme(s.clone())
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// authority   = [ userinfo "@" ] host [ ":" port ] (RFC 3986)
// userinfo    = *( unreserved / pct-encoded / sub-delims / ":" )
// host        = IP-literal / IPv4address / reg-name 
// port        = *DIGIT

#[derive(Debug, Clone)]
pub struct Authority {
    pub userinfo: String,
    pub host: String,
    pub port: u16,
}

impl Authority {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Authority {userinfo: String::new(),host: String::new(),port: 0}
        } else {

            let uinfore: Regex = Regex::new(r"((?<userinfo>[^/?#@]*)@)?").unwrap();
            let portre: Regex = Regex::new(r"[^?#/@:](:(?<port>[0-9]+))$").unwrap();

            let mut infostring = match uinfore.captures(s) {
                Some(res) => match res.name("userinfo") {
                    Some(string) => string.as_str().to_string(),
                    None => String::new(),
                },
                None => String::new(),
            };

            let userinfo: String = infostring.clone();

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

            let port: u16 = portstring.clone().parse::<u16>().unwrap();

            let final_s = if !portstring.is_empty() {
                portstring.insert(0, ':');
                mod_s.replace(portstring.as_str(), "")
            } else {
                mod_s.clone()
            };

            Authority { userinfo: userinfo, host: final_s.clone(), port: port }
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

#[derive(Debug, PartialEq, Clone)]
pub struct Path (String);

impl Path {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Path(String::new())
        } else {
            Path(s.clone())
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// query       = *( pchar / "/" / "?" )
#[derive(Debug, PartialEq, Clone)]
pub struct Query (String);

impl Query {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Query(String::new())
        } else {
            Query(s.clone())
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// fragment    = *( pchar / "/" / "?" )
#[derive(Debug, PartialEq, Clone)]
pub struct Fragment (String);

impl Fragment {
    pub fn new(s: &String) -> Self {
        if s.is_empty() {
            Fragment(String::new())
        } else {
            Fragment(s.clone())
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
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
        let authstring = urimatch!(s, "authority", &cap);
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
            self.scheme.0,
            self.authority.userinfo,
            self.authority.host,
            self.authority.port.to_string(),
            self.path.0,
            self.query.0,
            self.fragment.0,
        }
    }
}

