use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;
use chrono::NaiveDateTime;

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/authorities/authority_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AuthorityById;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/authorities/authority_with_requests_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AuthorityWithRequestsById;

pub async fn get_authority_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<authority_by_id::ResponseData, Box<dyn Error>> {

    let request_body = AuthorityById::build_query(authority_by_id::Variables {
        id,
    });

    let res = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", bearer))
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<authority_by_id::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "missing response data")
    })?;

    // serve HTML page with response_body
    Ok(response)
}

pub async fn get_authority_by_id_with_requests(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<authority_with_requests_by_id::ResponseData, Box<dyn Error>> {

    let request_body = AuthorityWithRequestsById::build_query(authority_with_requests_by_id::Variables {
        id: id.clone(),
    });

    let res = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", bearer.clone()))
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<authority_with_requests_by_id::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors in authority_with_requests query:");

        for error in &errors {
            println!("{:?}", error);
        }
    }

    let response = response_body.data.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "missing response data")
    })?;

    Ok(response)
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/authorities/all_authorities.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AllAuthorities;

pub async fn all_authorities(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_authorities::ResponseData, Box<dyn Error>> {

    let request_body = AllAuthorities::build_query(all_authorities::Variables {
    });

    let res = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", bearer))
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<all_authorities::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "missing response data")
    })?;

    // serve HTML page with response_body
    Ok(response)
}
