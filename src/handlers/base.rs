use actix_session::SessionExt;
use actix_web::{web, get, Responder, HttpResponse, HttpRequest};
use actix_identity::Identity;
use std::sync::Arc;

use crate::{generate_basic_context, AppData, graphql::authority::all_authorities};

#[get("/")]
pub async fn raw_index() -> impl Responder {
    return HttpResponse::Found().append_header(("Location", "/en")).finish()
}

#[get("/{lang}")]
pub async fn index(
    data: web::Data<AppData>,
    params: web::Path<String>,

    id: Option<Identity>,
    req: HttpRequest,
) -> impl Responder {

    let lang = params.into_inner();
    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    println!("Bearer: {:?}", &bearer);

    println!("Context: {:?}", &ctx);

    // When the user isn't logged in yet, we don't have a bearer token.
    // Render the page without authorities instead of panicking.
    if bearer.is_empty() {
        ctx.insert("authorities", &Vec::<serde_json::Value>::new());
    } else {
        match all_authorities(bearer, &data.api_url, Arc::clone(&data.client)).await {
            Ok(r) => ctx.insert("authorities", &r.authorities),
            Err(e) => {
                println!("Unable to get authorities: {:?}", e);
                ctx.insert("authorities", &Vec::<serde_json::Value>::new());
            }
        };
    }
      
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
