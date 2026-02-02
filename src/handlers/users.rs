// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use serde::{Deserialize};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct UserForm {
    user_name: String,
    email: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct AdminUserForm {
    user_name: String,
    email: String,
    role: String,
    validated: String,
}

use actix_session::{SessionExt};
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::{Identity};
use std::sync::Arc;


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_user_by_id};
use crate::graphql::user::{get_all_users, get_user_by_email};

/// Redirect /user/ to /profile
#[get("/{lang}/user/")]
pub async fn user_index_redirect(
    path: web::Path<String>,
) -> impl Responder {
    let lang = path.into_inner();
    HttpResponse::Found()
        .insert_header(("Location", format!("/{}/profile", lang)))
        .finish()
}

#[get("/{lang}/user/{user_id}")]
pub async fn user_by_id(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<(String, String)>,
    req:HttpRequest) -> impl Responder {

    let (lang, user_id) = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_user_by_id(user_id, bearer, &data.api_url, Arc::clone(&data.client))
        .await
        .expect("Unable to get user");

    ctx.insert("user", &r.user_by_id);

    let rendered = data.tmpl.render("users/user_page.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/person_by_name/{search_name}")]
pub async fn person_by_name(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, search_name) = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => {
            // Redirect to login if not authenticated
            return HttpResponse::Found()
                .insert_header(("Location", format!("/{}/log_in", lang)))
                .finish();
        }
    };

    let search_lower = search_name.to_lowercase();

    match get_all_users(bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(all_users_response) => {
            // Filter users by name (case-insensitive partial match)
            let matching_users: Vec<_> = all_users_response
                .all_users
                .into_iter()
                .filter(|user| user.name.to_lowercase().contains(&search_lower))
                .collect();

            ctx.insert("search_term", &search_name);
            ctx.insert("users", &matching_users);
            ctx.insert("result_count", &matching_users.len());

            let rendered = data.tmpl.render("users/search_results.html", &ctx).unwrap();
            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
        }
        Err(e) => {
            let error_str = e.to_string();
            println!("Error fetching users: {:?}", e);

            // Redirect to login if token expired or authentication error
            if error_str.contains("ExpiredSignature") || error_str.contains("Unauthorized") {
                return HttpResponse::Found()
                    .insert_header(("Location", format!("/{}/log_in", lang)))
                    .finish();
            }

            ctx.insert("search_term", &search_name);
            ctx.insert("error", &format!("Failed to fetch users: {}", e));
            ctx.insert("users", &Vec::<()>::new());
            ctx.insert("result_count", &0);

            let rendered = data.tmpl.render("users/search_results.html", &ctx).unwrap();
            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
        }
    }
}

#[get("/{lang}/profile")]
pub async fn user_profile(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let lang = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => {
            return HttpResponse::Found()
                .insert_header(("Location", format!("/{}/log_in", lang)))
                .finish();
        }
    };

    let email = match session.get::<String>("session_user").unwrap() {
        Some(s) => s,
        None => {
            return HttpResponse::Found()
                .insert_header(("Location", format!("/{}/log_in", lang)))
                .finish();
        }
    };

    match get_user_by_email(email, bearer, &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            ctx.insert("user", &response.user_by_email);

            let rendered = data.tmpl.render("users/profile.html", &ctx).unwrap();
            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
        }
        Err(e) => {
            let error_str = e.to_string();
            println!("Error fetching user profile: {:?}", e);

            if error_str.contains("ExpiredSignature") || error_str.contains("Unauthorized") {
                return HttpResponse::Found()
                    .insert_header(("Location", format!("/{}/log_in", lang)))
                    .finish();
            }

            ctx.insert("error", &format!("Failed to fetch profile: {}", e));
            let rendered = data.tmpl.render("users/profile.html", &ctx).unwrap();
            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
        }
    }
}
