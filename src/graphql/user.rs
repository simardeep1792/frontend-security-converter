use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;

// Type aliases for GraphQL scalar types
type UUID = String;
type NaiveDateTime = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/users/user_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UserById;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/users/all_users.graphql",
    response_derives = "Debug, Serialize, PartialEq, Clone"
)]
pub struct AllUsers;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/users/user_by_email.graphql",
    response_derives = "Debug, Serialize, PartialEq, Clone"
)]
pub struct UserByEmail;

pub async fn get_user_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<user_by_id::ResponseData, Box<dyn Error>> {

    let request_body = UserById::build_query(user_by_id::Variables {
        id,
    });

    let auth_bearer = format!("Bearer {}", bearer);

    let res = client
        .post(api_url)
        .header("Authorization", auth_bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<user_by_id::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data
        .expect("missing response data");

    // serve HTML page with response_body
    Ok(response)
}

pub async fn get_all_users(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_users::ResponseData, Box<dyn Error>> {
    let request_body = AllUsers::build_query(all_users::Variables {});

    let auth_bearer = format!("Bearer {}", bearer);

    let res = client
        .post(api_url)
        .header("Authorization", auth_bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<all_users::ResponseData> = res.json().await?;

    if let Some(errors) = &response_body.errors {
        println!("there are errors:");
        for error in errors {
            println!("{:?}", error);
        }
        // Return error instead of panicking
        let error_msg = errors.iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(error_msg.into());
    }

    let response = response_body.data
        .ok_or("missing response data")?;

    Ok(response)
}

pub async fn get_user_by_email(email: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<user_by_email::ResponseData, Box<dyn Error>> {
    let request_body = UserByEmail::build_query(user_by_email::Variables {
        email,
    });

    let auth_bearer = format!("Bearer {}", bearer);

    let res = client
        .post(api_url)
        .header("Authorization", auth_bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<user_by_email::ResponseData> = res.json().await?;

    if let Some(errors) = &response_body.errors {
        println!("there are errors:");
        for error in errors {
            println!("{:?}", error);
        }
        let error_msg = errors.iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(error_msg.into());
    }

    let response = response_body.data
        .ok_or("missing response data")?;

    Ok(response)
}