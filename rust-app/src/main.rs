use opensearch::http::transport::{SingleNodeConnectionPool, Transport, TransportBuilder};
use opensearch::http::Url;
use opensearch::indices::{IndicesCreateParts, IndicesDeleteParts};
use opensearch::{
    BulkParts, DeleteParts, IndexParts, OpenSearch, SearchParts, http::request::JsonBody,
};
use serde_json::{Value, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse("https://opensearch-node:9200")?;
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool).disable_proxy().build()?;
    let client = OpenSearch::new(transport);

    // Create an index
    let mut response = client
        .indices()
        .create(IndicesCreateParts::Index("movies"))
        .body(json!({
            "mappings" : {
                "properties" : {
                    "title" : { "type" : "text" }
                }
            }
        }))
        .send()
        .await?;

    let mut successful = response.status_code().is_success();

    if successful {
        println!("Successfully created an index");
    } else {
        println!("Could not create an index");
    }

    // Index a single document
    println!("Indexing a single document...");
    response = client
        .index(IndexParts::IndexId("movies", "1"))
        .body(json!({
            "id": 1,
            "title": "Moneyball",
            "director": "Bennett Miller",
            "year": "2011"
        }))
        .send()
        .await?;

    successful = response.status_code().is_success();

    if successful {
        println!("Successfully indexed a document");
    } else {
        println!("Could not index document");
    }

    // Index multiple documents using the bulk operation

    println!("Indexing multiple documents...");

    let mut body: Vec<JsonBody<_>> = Vec::with_capacity(4);

    // add the first operation and document
    body.push(json!({"index": {"_id": "2"}}).into());
    body.push(
        json!({
            "id": 2,
            "title": "Interstellar",
            "director": "Christopher Nolan",
            "year": "2014"
        })
        .into(),
    );

    // add the second operation and document
    body.push(json!({"index": {"_id": "3"}}).into());
    body.push(
        json!({
            "id": 3,
            "title": "Star Trek Beyond",
            "director": "Justin Lin",
            "year": "2015"
        })
        .into(),
    );

    response = client
        .bulk(BulkParts::Index("movies"))
        .body(body)
        .send()
        .await?;

    let mut response_body = response.json::<Value>().await?;
    successful = response_body["errors"].as_bool().unwrap() == false;

    if successful {
        println!("Successfully performed bulk operations");
    } else {
        println!("Could not perform bulk operations");
    }

    // Search for a document

    println!("Searching for a document...");
    response = client
        .search(SearchParts::Index(&["movies"]))
        .from(0)
        .size(10)
        .body(json!({
            "query": {
                "multi_match": {
                    "query": "miller",
                    "fields": ["title^2", "director"]
                }
            }
        }))
        .send()
        .await?;

    response_body = response.json::<Value>().await?;
    for hit in response_body["hits"]["hits"].as_array().unwrap() {
        // print the source document
        println!("{}", serde_json::to_string_pretty(&hit["_source"]).unwrap());
    }

    // Delete a document

    response = client
        .delete(DeleteParts::IndexId("movies", "2"))
        .send()
        .await?;

    successful = response.status_code().is_success();

    if successful {
        println!("Successfully deleted a document");
    } else {
        println!("Could not delete document");
    }

    // Delete the index

    response = client
        .indices()
        .delete(IndicesDeleteParts::Index(&["movies"]))
        .send()
        .await?;

    successful = response.status_code().is_success();

    if successful {
        println!("Successfully deleted the index");
    } else {
        println!("Could not delete the index");
    }

    Ok(())
}
