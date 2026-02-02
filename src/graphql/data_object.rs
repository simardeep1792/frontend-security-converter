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
    query_path = "queries/data_objects/all_data_objects.graphql",
    response_derives = "Debug, Serialize, PartialEq, Clone"
)]
pub struct AllDataObjects;

pub async fn all_data_objects(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_data_objects::ResponseData, Box<dyn Error>> {

    let request_body = AllDataObjects::build_query(all_data_objects::Variables {
    });

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<all_data_objects::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data
        .expect("missing response data");

    Ok(response)
}
