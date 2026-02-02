// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder, get, post, web};
use actix_session::{SessionExt};
use actix_identity::{Identity};

use crate::{AppData, generate_basic_context, graphql};
use crate::graphql::user::get_user_by_email;
use crate::graphql::authority::get_authorities_by_creator_id;

use super::LoginForm;

#[get("/{lang}/log_in")]
pub async fn login_handler(
    path: web::Path<String>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {

    let lang = path.into_inner();

    let session = req.get_session();

    let ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let rendered = data.tmpl.render("authentication/log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/log_in")]
pub async fn login_form_input(
    path: web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>,
    _id: Option<Identity>,
) -> impl Responder {

    let lang = path.into_inner();

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() {
        println!("Form is empty");
        return HttpResponse::Found().append_header(("Location", format!("/{}/log_in", &lang))).finish()
    };
    
    let login_data = graphql::login(
        form.email.to_lowercase().trim().to_string(),
        form.password.clone(), 
        &data.api_url,
        Arc::clone(&data.client),
    )
        .await
        .expect("Unable to login").sign_in;

    // Add user_name and role to session
    Identity::login(&req.extensions(), login_data.email.to_owned())
        .expect("Unable to login / identity");

    println!("{:?}", &login_data);

    let session = req.get_session();

    session.insert("role", login_data.role.to_owned())
        .expect("Unable to set role");

    session.insert("session_user", login_data.email.to_owned())
        .expect("Unable to set user name");

    session.insert("bearer", login_data.bearer.to_owned())
        .expect("Unable to set bearer");

    // Fetch user details to get user_id
    if let Ok(user_response) = get_user_by_email(
        login_data.email.clone(),
        login_data.bearer.clone(),
        &data.api_url,
        Arc::clone(&data.client),
    ).await {
        let user_id = user_response.user_by_email.id.clone();
        session.insert("user_id", user_id.clone())
            .expect("Unable to set user_id");

        // Fetch authorities created by this user
        if let Ok(authorities) = get_authorities_by_creator_id(
            user_id,
            login_data.bearer.clone(),
            &data.api_url,
            Arc::clone(&data.client),
        ).await {
            // If user has at least one authority, store the first one's ID
            if let Some(authority) = authorities.first() {
                session.insert("authority_id", authority.id.clone())
                    .expect("Unable to set authority_id");
                println!("User has authority: {} ({})", authority.name, authority.id);
            } else {
                println!("User has no authorities - may be admin");
            }
        }
    }

    return HttpResponse::Found()
        .append_header(("Location", "/"))
        .append_header(("Bearer", login_data.bearer))
        .finish()
}

#[get("/{lang}/log_out")]
pub async fn logout(
    path: web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let lang = path.into_inner();

    let session = req.get_session();

    session.clear();
    id.unwrap().logout();

    HttpResponse::Found().append_header(("Location", format!("/{}", &lang))).finish()
}