
use crate::HttpRequest;
use crate::HttpResponse;

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