use std::sync::Arc;

use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use actix_identity::Identity;
use serde::Serialize;

use crate::{AppData, generate_basic_context};
use crate::graphql::all_data_objects;

// Simplified struct to match available backend data
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MetadataForTemplate {
    id: String,
    domain: String,
    tags: Vec<String>,
    created_at: String,
    data_object: Option<DataObjectForTemplate>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DataObjectForTemplate {
    id: String,
    title: String,
    description: String,
}

#[get("/{lang}/metadata/tag/{tag}")]
pub async fn metadata_by_tag(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, tag) = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    // Fetch all data objects and filter by exact tag match
    let data_objects_response = all_data_objects(
        bearer,
        &data.api_url,
        Arc::clone(&data.client),
    )
    .await
    .expect("Unable to get data objects");

    // Filter by exact tag match and transform to template format
    // metadata is now a single Metadata object (not an array)
    // Tags are Vec<Option<String>> from GraphQL, so we need to handle the Option
    let metadata_list: Vec<MetadataForTemplate> = data_objects_response.data_objects
        .iter()
        .filter(|obj| {
            obj.metadata.tags.iter().any(|t| t.as_ref().map_or(false, |s| s == &tag))
        })
        .map(|obj| {
            MetadataForTemplate {
                id: obj.metadata.id.clone(),
                domain: obj.metadata.domain.clone(),
                tags: obj.metadata.tags.iter().filter_map(|t| t.clone()).collect(),
                created_at: obj.created_at.clone(),
                data_object: Some(DataObjectForTemplate {
                    id: obj.id.clone(),
                    title: obj.title.clone(),
                    description: obj.description.clone(),
                }),
            }
        })
        .collect();

    ctx.insert("tag", &tag);
    ctx.insert("metadata_list", &metadata_list);

    // Serialize metadata to JSON for JavaScript rendering
    let metadata_json = serde_json::to_string(&metadata_list)
        .expect("Failed to serialize metadata data");
    ctx.insert("metadata_json", &metadata_json);

    let rendered = data.tmpl.render("metadata/tag_results.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

#[get("/{lang}/search_metadata/pattern/{pattern}")]
pub async fn search_metadata(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, pattern) = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    // Fetch all data objects and filter by pattern (case-insensitive fuzzy match on tags)
    let data_objects_response = all_data_objects(
        bearer,
        &data.api_url,
        Arc::clone(&data.client),
    )
    .await
    .expect("Unable to get data objects");

    let pattern_lower = pattern.to_lowercase();

    // Filter by pattern match on tags (case-insensitive contains)
    // metadata is now a single Metadata object (not an array)
    // Tags are Vec<Option<String>> from GraphQL, so we need to handle the Option
    let metadata_list: Vec<MetadataForTemplate> = data_objects_response.data_objects
        .iter()
        .filter(|obj| {
            obj.metadata.tags.iter().any(|t| t.as_ref().map_or(false, |s| s.to_lowercase().contains(&pattern_lower)))
        })
        .map(|obj| {
            MetadataForTemplate {
                id: obj.metadata.id.clone(),
                domain: obj.metadata.domain.clone(),
                tags: obj.metadata.tags.iter().filter_map(|t| t.clone()).collect(),
                created_at: obj.created_at.clone(),
                data_object: Some(DataObjectForTemplate {
                    id: obj.id.clone(),
                    title: obj.title.clone(),
                    description: obj.description.clone(),
                }),
            }
        })
        .collect();

    ctx.insert("tag", &pattern);
    ctx.insert("metadata_list", &metadata_list);

    // Serialize metadata to JSON for JavaScript rendering
    let metadata_json = serde_json::to_string(&metadata_list)
        .expect("Failed to serialize metadata data");
    ctx.insert("metadata_json", &metadata_json);

    let rendered = data.tmpl.render("metadata/tag_results.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

#[get("/{lang}/metadata/domain/{domain}")]
pub async fn metadata_by_domain(
    data: web::Data<AppData>,
    id: Option<Identity>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (lang, domain) = path.into_inner();

    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    // Fetch all data objects and filter by domain
    let data_objects_response = all_data_objects(
        bearer,
        &data.api_url,
        Arc::clone(&data.client),
    )
    .await
    .expect("Unable to get data objects");

    // Filter by domain and transform to template format
    // metadata is now a single Metadata object (not an array)
    // Tags are Vec<Option<String>> from GraphQL, so we need to handle the Option
    let metadata_list: Vec<MetadataForTemplate> = data_objects_response.data_objects
        .iter()
        .filter(|obj| obj.metadata.domain == domain)
        .map(|obj| {
            MetadataForTemplate {
                id: obj.metadata.id.clone(),
                domain: obj.metadata.domain.clone(),
                tags: obj.metadata.tags.iter().filter_map(|t| t.clone()).collect(),
                created_at: obj.created_at.clone(),
                data_object: Some(DataObjectForTemplate {
                    id: obj.id.clone(),
                    title: obj.title.clone(),
                    description: obj.description.clone(),
                }),
            }
        })
        .collect();

    ctx.insert("domain", &domain);
    ctx.insert("metadata_list", &metadata_list);

    // Serialize metadata to JSON for JavaScript rendering
    let metadata_json = serde_json::to_string(&metadata_list)
        .expect("Failed to serialize metadata data");
    ctx.insert("metadata_json", &metadata_json);

    let rendered = data.tmpl.render("metadata/domain_results.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}
