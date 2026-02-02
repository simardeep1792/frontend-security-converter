use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;

// Type aliases for GraphQL scalar types
type UUID = String;
type NaiveDateTime = String;

/// Input struct matching the new SubmitConversionRequestInput from schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitConversionInput {
    pub authority_id: String,
    pub data_object_title: String,
    pub data_object_description: String,
    pub metadata_domain: String,
    pub metadata_tags: Vec<String>,
    pub source_nation_code: String,
    pub target_nation_codes: Vec<String>,
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/conversion_request.graphql",
    response_derives = "Debug, Serialize, PartialEq, Clone"
)]
pub struct SubmitConversion;

pub async fn submit_conversion_request(
    input: SubmitConversionInput,
    api_url: &str,
    client: Arc<Client>,
    bearer: String,
) -> Result<submit_conversion::ResponseData, Box<dyn Error>> {

    let gql_input = submit_conversion::SubmitConversionRequestInput {
        authority_id: input.authority_id,
        data_object_title: input.data_object_title,
        data_object_description: input.data_object_description,
        metadata_domain: input.metadata_domain,
        metadata_tags: input.metadata_tags,
        source_nation_code: input.source_nation_code,
        target_nation_codes: input.target_nation_codes,
    };

    let request_body = SubmitConversion::build_query(submit_conversion::Variables {
        input: gql_input,
    });

    let res = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", bearer))
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<submit_conversion::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("GraphQL errors:");
        for error in &errors {
            println!("{:?}", error);
        }
        return Err(format!("GraphQL error: {:?}", errors).into());
    }

    let response = response_body.data
        .ok_or("Missing response data from submitConversionRequest")?;

    println!("Conversion request submitted successfully: {:?}", &response);

    Ok(response)
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/conversion_requests/conversion_request_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct ConversionRequestById;

pub async fn get_conversion_request_by_id(id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<conversion_request_by_id::ResponseData, Box<dyn Error>> {

    let request_body = ConversionRequestById::build_query(conversion_request_by_id::Variables {
        id,
    });

    let res = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", bearer))
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<conversion_request_by_id::ResponseData> = res.json().await?;

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
