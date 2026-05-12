use std::sync::Arc;

use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::{Identity};


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_authority_by_id, get_authority_by_id_with_requests};

#[get("/{lang}/authority/{authority_id}")]
pub async fn authority_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {
    let (lang, authority_id) = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    match get_authority_by_id_with_requests(authority_id.clone(), bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(v) => ctx.insert("authority", &v.authority_by_id),
        Err(e) => {
            println!("Authority detailed query failed, falling back: {:?}", e);
            let basic = match get_authority_by_id(authority_id, bearer, &data.api_url, Arc::clone(&data.client)).await {
                Ok(b) => b,
                Err(inner) => {
                    println!("Unable to get authority: {:?}", inner);
                    return HttpResponse::InternalServerError().body("Unable to load authority");
                }
            };
            ctx.insert("authority", &basic.authority_by_id);
        }
    }

    let rendered = data.tmpl.render("authority/authority.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
