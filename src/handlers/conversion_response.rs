use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_session::{SessionExt};
use actix_identity::{Identity};
use serde::{Serialize, Deserialize};
use chrono::{NaiveDateTime};
use schemars::JsonSchema;

use uuid::Uuid;

use crate::{AppData, generate_basic_context, graphql::{get_authority_by_id, submit_conversion_request, SubmitConversionInput}};

#[derive(Deserialize, Debug, Serialize)]
pub struct DocumentSubmissionForm {

    // Selected authority (for users without pre-assigned authority)
    #[serde(rename = "selected_authority_id")]
    pub selected_authority_id: Option<String>,

    #[serde(rename = "securityClassificationLevel")]
    pub security_classification_level: String,

    // Target nations for checkbox form processing - NATO members + AUS/NZL
    pub alb: Option<String>,
    pub aus: Option<String>,
    pub bel: Option<String>,
    pub bgr: Option<String>,
    pub can: Option<String>,
    pub hrv: Option<String>,
    pub cze: Option<String>,
    pub dnk: Option<String>,
    pub est: Option<String>,
    pub fin: Option<String>,
    pub fra: Option<String>,
    pub deu: Option<String>,
    pub grc: Option<String>,
    pub hun: Option<String>,
    pub isl: Option<String>,
    pub ita: Option<String>,
    pub lva: Option<String>,
    pub ltu: Option<String>,
    pub lux: Option<String>,
    pub mne: Option<String>,
    pub nld: Option<String>,
    pub nzl: Option<String>,
    pub mkd: Option<String>,
    pub nor: Option<String>,
    pub pol: Option<String>,
    pub prt: Option<String>,
    pub rou: Option<String>,
    pub svk: Option<String>,
    pub svn: Option<String>,
    pub esp: Option<String>,
    pub swe: Option<String>,
    pub tur: Option<String>,
    pub gbr: Option<String>,
    pub usa: Option<String>,

    // Releasable to organizations - checkbox form processing
    pub nato: Option<String>,
    pub eu: Option<String>,
    pub un: Option<String>,
    pub fvey: Option<String>,
    pub aukus: Option<String>,
    pub quad: Option<String>,

    #[serde(rename = "disclosureCategory")]
    pub disclosure_category: String,

    // Handling restrictions - checkbox form processing
    pub cui: Option<String>,
    pub fouo: Option<String>,
    pub les: Option<String>,
    pub sbu: Option<String>,
    pub noforn: Option<String>,
    pub propin: Option<String>,
    pub orcon: Option<String>,

    #[serde(rename = "handlingAuthority")]
    pub handling_authority: String,

    pub content: String,
}
/// The JSON formatted data payload submitted to the API that triggers
/// a security classification conversion
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InsertableConversionRequest {
    pub user_id: String,
    pub authority_id: String,
    pub data_object: InsertableDataObject,
    pub metadata: InsertableMetadata,
    pub source_nation_classification: String,
    pub source_nation_code: String,
    pub target_nation_codes: Vec<String>,
}

/// A lightweight struct to accept JSON formatted data from a ConversionRequest
/// needed to create a NewDataObject
/// GraphQL input type accepts plain String (will be encrypted internally)
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InsertableDataObject {
    pub title: String,        // GraphQL input as plain String
    pub description: String,   // GraphQL input as plain String
}

// Note: DataObjectInput type no longer exists in the new schema
// The submitConversionRequest mutation uses SubmitConversionRequestInput directly

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LLMFields {
    // Document Title
    pub title: String,
    // Two sentence description of the document
    pub description: String,
    // Single domain that best represents the content
    pub domain: Domain,
    // Up to six tags that help improve understanding the context
    pub tags: Vec<Option<String>>,

    // Identifier == unique home organization identifier for document
    pub identifier: String,

    // Authorization Reference - will be encrypted
    // References the authority document under which release is permitted.
    pub authorization_reference: Option<String>,

    // Release restrictions
    pub releasable_to_countries: Option<Vec<Option<String>>>,
    pub releasable_to_organizations: Option<Vec<Option<String>>>,
    pub releasable_to_categories: Option<Vec<Option<String>>>,
    pub disclosure_category: Option<String>,

    // Handling Restrictions
    pub handling_restrictions: Option<Vec<Option<String>>>,
    pub handling_authority: Option<String>,
    pub no_handling_restrictions: Option<bool>,
}

/// A light struct to accept the JSON formatted Metadata included with
/// a ConversionRequest
/// GraphQL input type accepts plain String (will be encrypted internally)
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InsertableMetadata {
    // Global Identifier
    pub identifier: String,

    // Authorization Reference - will be encrypted
    pub authorization_reference: Option<String>,
    pub authorization_reference_date: Option<NaiveDateTime>,

    // Originator and Custodian
    pub originator_organization_id: Uuid, // Authority
    pub custodian_organization_id: Uuid, // Authority

    // Format
    pub format: String,
    pub format_size: Option<i64>,

    // Safeguarding and Securing
    pub security_classification: String,

    // Disclosure & Releasability
    pub releasable_to_countries: Option<Vec<Option<String>>>,
    pub releasable_to_organizations: Option<Vec<Option<String>>>,
    pub releasable_to_categories: Option<Vec<Option<String>>>,
    pub disclosure_category: Option<String>,

    // Handling Restrictions
    pub handling_restrictions: Option<Vec<Option<String>>>,
    pub handling_authority: Option<String>,
    pub no_handling_restrictions: Option<bool>,

    // Legacy fields
    pub domain: Domain,
    pub tags: Vec<Option<String>>,
}

// Note: MetadataInput type no longer exists in the new schema
// The submitConversionRequest mutation uses SubmitConversionRequestInput directly

#[derive(JsonSchema, Deserialize, Debug, Clone, Serialize)]
pub enum Domain {
    INTEL,
    CYBER,
    OPERATIONS,
    LOGISTICS,
    COMMUNICATIONS,
    NUCLEAR,
    COUNTERTERRORISM,
    MARITIME,
    AEROSPACE,
    SPECIALOPS,
}

#[post("/{lang}/submit_document")]
pub async fn submit_document(
    path: web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest,
    form: web::Form<DocumentSubmissionForm>,
    id: Option<Identity>,
) -> impl Responder {

    println!("=== SUBMIT_DOCUMENT HANDLER CALLED ===");

    let lang = path.into_inner();

    let session = req.get_session();

    println!("Form content length: {}", form.content.len());
    println!("Selected authority_id from form: {:?}", form.selected_authority_id);

    // validate form has data or re-load form
    if form.content.is_empty() {
        println!("Form is empty - redirecting");
        return HttpResponse::Found().append_header(("Location", format!("/{}", &lang))).finish()
    };

    let _ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let user_id = match req.get_session().get::<String>("user_id").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    // Get authority_id from session, or from form if user selected one
    let authority_id = match req.get_session().get::<String>("authority_id").unwrap() {
        Some(s) if !s.is_empty() => s,
        _ => {
            // Try to get from form submission (user selected an authority)
            match &form.selected_authority_id {
                Some(id) if !id.is_empty() => id.clone(),
                _ => {
                    println!("No authority_id in session or form");
                    return HttpResponse::Found()
                        .append_header(("Location", format!("/{}/conversion_request?error=no_authority_selected", &lang)))
                        .finish();
                }
            }
        }
    };

    println!("Fetching authority with id: {}", authority_id);

    let authority = match get_authority_by_id(authority_id.clone(), bearer.clone(), &data.api_url, Arc::clone(&data.client)).await {
        Ok(response) => {
            println!("Authority fetched successfully: {}", response.authority_by_id.name);
            response.authority_by_id
        }
        Err(e) => {
            println!("Error fetching authority: {:?}", e);
            return HttpResponse::Found()
                .append_header(("Location", format!("/{}/conversion_request?error=authority_fetch_failed", &lang)))
                .finish();
        }
    };

    // Handle form - target nations
    let mut target_nations = Vec::new();

    if form.alb != None { target_nations.push("ALB".to_owned());};
    if form.aus != None { target_nations.push("AUS".to_owned());};
    if form.bel != None { target_nations.push("BEL".to_owned());};
    if form.bgr != None { target_nations.push("BGR".to_owned());};
    if form.can != None { target_nations.push("CAN".to_owned());};
    if form.hrv != None { target_nations.push("HRV".to_owned());};
    if form.cze != None { target_nations.push("CZE".to_owned());};
    if form.dnk != None { target_nations.push("DNK".to_owned());};
    if form.est != None { target_nations.push("EST".to_owned());};
    if form.fin != None { target_nations.push("FIN".to_owned());};
    if form.fra != None { target_nations.push("FRA".to_owned());};
    if form.deu != None { target_nations.push("DEU".to_owned());};
    if form.grc != None { target_nations.push("GRC".to_owned());};
    if form.hun != None { target_nations.push("HUN".to_owned());};
    if form.isl != None { target_nations.push("ISL".to_owned());};
    if form.ita != None { target_nations.push("ITA".to_owned());};
    if form.lva != None { target_nations.push("LVA".to_owned());};
    if form.ltu != None { target_nations.push("LTU".to_owned());};
    if form.lux != None { target_nations.push("LUX".to_owned());};
    if form.mne != None { target_nations.push("MNE".to_owned());};
    if form.nld != None { target_nations.push("NLD".to_owned());};
    if form.nzl != None { target_nations.push("NZL".to_owned());};
    if form.mkd != None { target_nations.push("MKD".to_owned());};
    if form.nor != None { target_nations.push("NOR".to_owned());};
    if form.pol != None { target_nations.push("POL".to_owned());};
    if form.prt != None { target_nations.push("PRT".to_owned());};
    if form.rou != None { target_nations.push("ROU".to_owned());};
    if form.svk != None { target_nations.push("SVK".to_owned());};
    if form.svn != None { target_nations.push("SVN".to_owned());};
    if form.esp != None { target_nations.push("ESP".to_owned());};
    if form.swe != None { target_nations.push("SWE".to_owned());};
    if form.tur != None { target_nations.push("TUR".to_owned());};
    if form.gbr != None { target_nations.push("GBR".to_owned());};
    if form.usa != None { target_nations.push("USA".to_owned());};

    // Handle form - releasable to organizations
    let mut releasable_orgs = Vec::new();

    if form.nato != None { releasable_orgs.push("NATO".to_owned());};
    if form.eu != None { releasable_orgs.push("EU".to_owned());};
    if form.un != None { releasable_orgs.push("UN".to_owned());};
    if form.fvey != None { releasable_orgs.push("FVEY".to_owned());};
    if form.aukus != None { releasable_orgs.push("AUKUS".to_owned());};
    if form.quad != None { releasable_orgs.push("QUAD".to_owned());};

    // Handle form - handling restrictions
    let mut handling_restrictions = Vec::new();

    if form.cui != None { handling_restrictions.push("CUI".to_owned());};
    if form.fouo != None { handling_restrictions.push("FOUO".to_owned());};
    if form.les != None { handling_restrictions.push("LES".to_owned());};
    if form.sbu != None { handling_restrictions.push("SBU".to_owned());};
    if form.noforn != None { handling_restrictions.push("NOFORN".to_owned());};
    if form.propin != None { handling_restrictions.push("PROPIN".to_owned());};
    if form.orcon != None { handling_restrictions.push("ORCON".to_owned());};

    // Use LLM to generate DataObject
    // Build the prompt for structured JSON extraction
    let prompt = format!(
        "Extract security metadata from this document as valid JSON.\n\n\
        Document: {}\n\n\
        Requirements:\n\
        - title: Clear descriptive title (required string)\n\
        - description: 2-sentence summary (required string)\n\
        - domain: One of INTEL, CYBER, OPERATIONS, LOGISTICS, COMMUNICATIONS, NUCLEAR, COUNTERTERRORISM, MARITIME, AEROSPACE, SPECIALOPS\n\
        - tags: Array of 3-6 classification tags (required array of strings)\n\
        - identifier: Unique ID in format ORG-DOMAIN-DATE-XXXX (required string)\n\
        - For optional arrays: use empty array [] if no values, never null\n\
        - For optional strings: use null if no value\n\n\
        Output valid JSON matching this schema:\n\
        {{\n\
          \"title\": \"string\",\n\
          \"description\": \"string\",\n\
          \"domain\": \"INTEL|CYBER|OPERATIONS|LOGISTICS|COMMUNICATIONS|NUCLEAR|COUNTERTERRORISM|MARITIME|AEROSPACE|SPECIALOPS\",\n\
          \"tags\": [\"string\"],\n\
          \"identifier\": \"string\",\n\
          \"authorization_reference\": \"string or null\",\n\
          \"releasable_to_countries\": [\"string\"] or null,\n\
          \"releasable_to_organizations\": [\"string\"] or null,\n\
          \"releasable_to_categories\": [\"string\"] or null,\n\
          \"disclosure_category\": \"string or null\",\n\
          \"handling_restrictions\": [\"string\"] or null,\n\
          \"handling_authority\": \"string or null\",\n\
          \"no_handling_restrictions\": true/false or null\n\
        }}\n\n\
        Output valid JSON only:",
        &form.content
    );

    let llm = &data.llm;

    println!("Starting LLM generation using {} provider", llm.provider_name());

    let llm_response = llm.generate(&prompt, None)
        .await
        .expect("Unable to retrieve LLM generated content");

    let llm_fields: LLMFields = serde_json::from_str(&llm_response.content)
        .expect("Unable to derive LLMfields from LLM response");

    if let Some(duration) = llm_response.duration_ms {
        println!("LLM Generation Completed in {} ms using model {}", duration, llm_response.model);
    } else {
        println!("LLM Generation Completed using model {}", llm_response.model);
    }

    let data_struct: InsertableDataObject = InsertableDataObject { 
        title: llm_fields.title, 
        description: llm_fields.description, 
    };

    println!("{:?}", &data_struct);

    let custodian_organization_id = Uuid::parse_str(&authority.id).expect("Unable to convert Str to UUID");
    let originator_organization_id = Uuid::parse_str(&authority.id).expect("Unable to convert Str to UUID");
    let security_classification = form.security_classification_level.clone();

    let today = chrono::Utc::now().naive_utc();

    let meta_struct: InsertableMetadata = InsertableMetadata { 
        identifier: llm_fields.identifier, 
        authorization_reference: llm_fields.authorization_reference, 
        authorization_reference_date: Some(today), 
        originator_organization_id: custodian_organization_id, 
        custodian_organization_id: originator_organization_id, 
        format: "Markdown".to_string(), 
        format_size: Some(form.content.len() as i64), 
        security_classification: security_classification, 
        releasable_to_countries: llm_fields.releasable_to_countries, 
        releasable_to_organizations: llm_fields.releasable_to_organizations, 
        releasable_to_categories: llm_fields.releasable_to_categories, 
        disclosure_category: llm_fields.disclosure_category, 
        handling_restrictions: llm_fields.handling_restrictions, 
        handling_authority: llm_fields.handling_authority, 
        no_handling_restrictions: llm_fields.no_handling_restrictions, 
        domain: llm_fields.domain, 
        tags: llm_fields.tags,
    };
    
    println!("{:?}", &meta_struct);

    println!("Structured Data Successfully Generated");

    // Integrate into full ConversionRequest 

    let conversion_request = InsertableConversionRequest {
        user_id: user_id,
        authority_id: authority.id,
        data_object: data_struct,
        metadata: meta_struct,
        source_nation_classification: form.security_classification_level.clone(),
        source_nation_code: authority.nation.nation_code,
        target_nation_codes: target_nations,
    };

    println!("{:?}", conversion_request);

    // Store conversion_request in session for validation page
    session.insert("conversion_request", &conversion_request)
        .expect("Unable to store conversion_request in session");

    // Redirect to validation page
    HttpResponse::Found()
        .append_header(("Location", format!("/{}/validate_conversion", &lang)))
        .finish()
}

/// Display the validation page with generated conversion request data
#[get("/{lang}/validate_conversion")]
pub async fn validate_conversion(
    path: web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    // Retrieve conversion_request from session
    let conversion_request: InsertableConversionRequest = match session.get("conversion_request").unwrap() {
        Some(cr) => cr,
        None => {
            // If no conversion request in session, redirect back to form
            return HttpResponse::Found()
                .append_header(("Location", format!("/{}/conversion_request", &lang)))
                .finish();
        }
    };

    // Prepare context for template
    ctx.insert("user_id", &conversion_request.user_id);
    ctx.insert("authority_id", &conversion_request.authority_id);
    ctx.insert("data_object", &conversion_request.data_object);
    ctx.insert("metadata", &conversion_request.metadata);
    ctx.insert("source_classification", &conversion_request.source_nation_classification);
    ctx.insert("source_nation_code", &conversion_request.source_nation_code);
    ctx.insert("target_nations", &conversion_request.target_nation_codes);

    // Convert Optional Vec fields to comma-separated strings for display
    let tags = conversion_request.metadata.tags
        .iter()
        .filter_map(|t| t.as_ref())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    ctx.insert("tags", &tags);

    let releasable_to_countries = conversion_request.metadata.releasable_to_countries
        .unwrap_or_default()
        .iter()
        .filter_map(|t| t.as_ref())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    ctx.insert("releasable_to_countries", &releasable_to_countries);

    let releasable_to_organizations = conversion_request.metadata.releasable_to_organizations
        .unwrap_or_default()
        .iter()
        .filter_map(|t| t.as_ref())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    ctx.insert("releasable_to_organizations", &releasable_to_organizations);

    let releasable_to_categories = conversion_request.metadata.releasable_to_categories
        .unwrap_or_default()
        .iter()
        .filter_map(|t| t.as_ref())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    ctx.insert("releasable_to_categories", &releasable_to_categories);

    let handling_restrictions = conversion_request.metadata.handling_restrictions
        .unwrap_or_default()
        .iter()
        .filter_map(|t| t.as_ref())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    ctx.insert("handling_restrictions", &handling_restrictions);

    let rendered = data.tmpl.render("conversion_request/validate_conversion.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Form data from the validation page
#[derive(Deserialize, Debug)]
pub struct ValidationForm {
    // Data Object
    pub title: String,
    pub description: String,

    // Metadata
    pub identifier: String,
    pub domain: String,
    pub tags: String,
    pub authorization_reference: Option<String>,
    pub disclosure_category: Option<String>,
    pub handling_authority: Option<String>,
    pub releasable_to_countries: String,
    pub releasable_to_organizations: String,
    pub releasable_to_categories: String,
    pub handling_restrictions: String,
    pub no_handling_restrictions: Option<String>,

    // Hidden fields (read-only)
    pub user_id: String,
    pub authority_id: String,
    pub source_nation_classification: String,
    pub source_nation_code: String,
    pub target_nation_codes: String,
    pub format: String,
    pub format_size: Option<i64>,
    pub security_classification: String,
    pub originator_organization_id: String,
    pub custodian_organization_id: String,
    pub authorization_reference_date: Option<String>,
}

/// Accept validated data and submit to API
#[post("/{lang}/confirm_conversion")]
pub async fn confirm_conversion(
    path: web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest,
    form: web::Form<ValidationForm>,
    id: Option<Identity>,
) -> impl Responder {
    let lang = path.into_inner();
    let session = req.get_session();

    let mut ctx = generate_basic_context(id, &lang, req.uri().path(), &session);

    let bearer = match session.get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    // Parse target_nation_codes back to Vec<String>
    let target_nations: Vec<String> = form.target_nation_codes
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Parse tags from comma-separated string
    let tags: Vec<String> = form.tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Build the simplified SubmitConversionInput for the new API
    let conversion_input = SubmitConversionInput {
        authority_id: form.authority_id.clone(),
        data_object_title: form.title.clone(),
        data_object_description: form.description.clone(),
        metadata_domain: form.domain.clone(),
        metadata_tags: tags,
        source_nation_code: form.source_nation_code.clone(),
        target_nation_codes: target_nations,
    };

    println!("Submitting conversion request: {:?}", conversion_input);

    // Submit to API
    match submit_conversion_request(
        conversion_input,
        &data.api_url,
        Arc::clone(&data.client),
        bearer
    )
    .await {
        Ok(response) => {
            // Clear session data
            session.remove("conversion_request");

            // Generate Response for User
            ctx.insert("conversion_response", &response);

            let rendered = data.tmpl.render("conversion_request/conversion_response.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        }
        Err(e) => {
            // Handle the error - the submitConversionRequest mutation may not exist
            println!("Error submitting conversion request: {:?}", e);
            ctx.insert("error", &format!("Unable to submit conversion request: {}", e));

            let rendered = data.tmpl.render("conversion_request/conversion_error.html", &ctx).unwrap();
            HttpResponse::InternalServerError().body(rendered)
        }
    }
}

