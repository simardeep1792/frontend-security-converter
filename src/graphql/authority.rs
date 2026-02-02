use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;

// Import the AllNations query from the nation module to avoid duplication
use super::nation::{all_nations, AllNations};

type UUID = String;

/// Authority structure that matches what the templates expect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityData {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub nation: NationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NationData {
    pub id: String,
    pub nation_code: String,
    pub nation_name: String,
}

/// Response wrapper to match the expected interface
pub struct AuthorityByIdResponse {
    pub authority_by_id: AuthorityData,
}

pub async fn get_authority_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<AuthorityByIdResponse, Box<dyn Error>> {
    use graphql_client::{GraphQLQuery, Response};

    let request_body = AllNations::build_query(all_nations::Variables {});

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<all_nations::ResponseData> = res.json().await?;

    if let Some(errors) = &response_body.errors {
        println!("there are errors:");
        for error in errors {
            println!("{:?}", error);
        }
    }

    let data = response_body.data
        .ok_or("missing response data")?;

    // Search through all nations to find the authority by ID
    for nation in data.nations {
        for authority in nation.authorities {
            if authority.id == id {
                return Ok(AuthorityByIdResponse {
                    authority_by_id: AuthorityData {
                        id: authority.id,
                        name: authority.name,
                        email: authority.email,
                        phone: authority.phone,
                        nation: NationData {
                            id: nation.id.clone(),
                            nation_code: nation.nation_code.clone(),
                            nation_name: nation.nation_name.clone(),
                        },
                    },
                });
            }
        }
    }

    Err(format!("Authority with id {} not found", id).into())
}

// Reuse the AllNations query for all_authorities
pub async fn all_authorities(bearer: String, api_url: &str, client: Arc<Client>) -> Result<Vec<AuthorityData>, Box<dyn Error>> {
    use graphql_client::{GraphQLQuery, Response};

    let request_body = AllNations::build_query(all_nations::Variables {});

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<all_nations::ResponseData> = res.json().await?;

    if let Some(errors) = &response_body.errors {
        println!("there are errors:");
        for error in errors {
            println!("{:?}", error);
        }
    }

    let data = response_body.data
        .ok_or("missing response data")?;

    // Flatten all authorities from all nations
    let mut authorities = Vec::new();
    for nation in data.nations {
        for authority in nation.authorities {
            authorities.push(AuthorityData {
                id: authority.id,
                name: authority.name,
                email: authority.email,
                phone: authority.phone,
                nation: NationData {
                    id: nation.id.clone(),
                    nation_code: nation.nation_code.clone(),
                    nation_name: nation.nation_name.clone(),
                },
            });
        }
    }

    Ok(authorities)
}

/// Get authorities created by a specific user
pub async fn get_authorities_by_creator_id(creator_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<Vec<AuthorityData>, Box<dyn Error>> {
    use graphql_client::{GraphQLQuery, Response};

    let request_body = AllNations::build_query(all_nations::Variables {});

    let res = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", bearer))
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<all_nations::ResponseData> = res.json().await?;

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

    let data = response_body.data
        .ok_or("missing response data")?;

    // Find authorities created by this user
    let mut authorities = Vec::new();
    for nation in data.nations {
        for authority in nation.authorities {
            if authority.creator_id == creator_id {
                authorities.push(AuthorityData {
                    id: authority.id,
                    name: authority.name,
                    email: authority.email,
                    phone: authority.phone,
                    nation: NationData {
                        id: nation.id.clone(),
                        nation_code: nation.nation_code.clone(),
                        nation_name: nation.nation_name.clone(),
                    },
                });
            }
        }
    }

    Ok(authorities)
}
