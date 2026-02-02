use std::sync::Arc;

use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::Identity;
use serde::Serialize;

use crate::{AppData, generate_basic_context};
use crate::graphql::nation::all_nations;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorityForTemplate {
    id: String,
    name: String,
    email: String,
    phone: String,
    created_at: String,
    updated_at: String,
    expires_at: Option<String>,
    nation: NationForTemplate,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NationForTemplate {
    id: String,
    nation_name: String,
    nation_code: String,
}

#[get("/{lang}/authority/{authority_id}")]
pub async fn authority_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, authority_id) = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    // Fetch all nations and find the authority within
    let nations_response = all_nations(bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get nations");

    // Find the authority by ID across all nations
    let mut found_authority: Option<AuthorityForTemplate> = None;
    for nation in &nations_response.nations {
        for auth in &nation.authorities {
            if auth.id == authority_id {
                found_authority = Some(AuthorityForTemplate {
                    id: auth.id.clone(),
                    name: auth.name.clone(),
                    email: auth.email.clone(),
                    phone: auth.phone.clone(),
                    created_at: auth.created_at.to_string(),
                    updated_at: auth.updated_at.to_string(),
                    expires_at: auth.expires_at.as_ref().map(|dt| format!("{:?}", dt)),
                    nation: NationForTemplate {
                        id: nation.id.clone(),
                        nation_name: nation.nation_name.clone(),
                        nation_code: nation.nation_code.clone(),
                    },
                });
                break;
            }
        }
        if found_authority.is_some() {
            break;
        }
    }

    match found_authority {
        Some(authority) => {
            ctx.insert("authority", &authority);
            let rendered = data.tmpl.render("authority/authority.html", &ctx).unwrap();
            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
        }
        None => {
            HttpResponse::NotFound().body("Authority not found")
        }
    }
}

