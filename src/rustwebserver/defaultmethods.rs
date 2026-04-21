
use crate::HttpRequest;
use crate::HttpResponse;

/*
    let (status_line, filename) = match http_request[0].as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(error) => panic!("Error reading file: {error:?}"),
    };
    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    ); 
*/

pub fn handle_get(req: HttpRequest) -> HttpResponse {
    let res = HttpResponse::new();



    return res;
}

pub fn handle_head(req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_options(req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_trace(req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}