use actix_session::SessionExt;
use actix_web::{web, get, Responder, HttpResponse, HttpRequest};
use actix_identity::Identity;
use std::sync::Arc;
use serde::Serialize;

use crate::{generate_basic_context, AppData, graphql::nation::all_nations};

#[derive(Serialize)]
struct AuthorityWithNation {
    id: String,
    name: String,
    email: String,
    phone: String,
    nation: NationInfo,
}

#[derive(Serialize, Clone)]
struct NationInfo {
    id: String,
    nation_name: String,
    nation_code: String,
}

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

    let r = all_nations(bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get nations");

    // Flatten nations into authorities with nation info (matching template expectations)
    let authorities: Vec<AuthorityWithNation> = r.nations
        .iter()
        .flat_map(|nation| {
            let nation_info = NationInfo {
                id: nation.id.clone(),
                nation_name: nation.nation_name.clone(),
                nation_code: nation.nation_code.clone(),
            };
            nation.authorities.iter().map(move |auth| {
                AuthorityWithNation {
                    id: auth.id.clone(),
                    name: auth.name.clone(),
                    email: auth.email.clone(),
                    phone: auth.phone.clone(),
                    nation: nation_info.clone(),
                }
            })
        })
        .collect();

    ctx.insert("authorities", &authorities);

    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}