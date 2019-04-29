use crate::sync::server::{TkzrServer, DEFAULT_PORT};

use crate::sync::USetOp;
use crate::sync::test_utils::example_replica_1;

use reqwest::StatusCode;

use tempfile::{tempdir_in, TempDir};

fn make_test_db_dir() -> TempDir {
    let res = std::fs::create_dir("/tmp/tkzr");
    if let Err(e) = res {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("error creating test db dir: {}", e);
        }
    }

    tempdir_in("/tmp/tkzr").expect("Failed to open tempdir in /tmp/tkzr")
}


#[test]
/// Smoke test, check server starts up and shuts down without error
fn test_sync_server() {
    let db_dir = make_test_db_dir();

    let server = TkzrServer::test_server(&db_dir, DEFAULT_PORT);

    let res = server.start();
    // TODO: when https://github.com/actix/actix-net/pull/16 is merged into actix-web (i.e. its
    // dependency is upgraded) we can derive Debug on TkzrServerHandle and then just do
    // `assert!(res.is_ok(), "Failed to start server: {}", res.unwrap_err());`
    if let Err(e) = res {
        assert!(false, "Failed to start server: {}", e); 
    }
    else {
        let server_handle = res.unwrap();
        server_handle.shutdown();
    }
}


#[test]
/// Check that trying to bind to a socket/ip address+port that's already in use fails.
fn test_sync_server_bind_in_use_fails() {
    let port = DEFAULT_PORT+1;

    let db_dir = make_test_db_dir();

    let server = TkzrServer::test_server(&db_dir, port);
    let server_handle = server.start().expect("Failed to start server");

    // start another server with same port
    let server2 = TkzrServer::test_server("/dev/null", port);

    let res = server2.start();
    assert!(res.is_err(), "No error starting server on port already in use");

    let err = res.err().unwrap();
    assert!(err.to_string().contains("Failed to bind to address"),
        "Incorrect error when binding to address in use: {}", err);

    server_handle.shutdown();
}

#[test]
/// Check that server responds with 404 to queries at root
fn test_sync_server_404s_at_root() {
    let port = DEFAULT_PORT+2;

    let db_dir = make_test_db_dir();

    let server = TkzrServer::test_server(&db_dir, port);
    let server_handle = server.start().expect("Failed to start server");
   

    let health_url = format!("http://localhost:{}/", port);

    let res = reqwest::get(&health_url);
    assert!(res.is_ok(), "Error sending get request to /: {}", res.unwrap_err());

    let response = res.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    server_handle.shutdown();
}


#[test]
/// Check that server responds to queries at /health
fn test_sync_server_health() {
    let port = DEFAULT_PORT+3;

    let db_dir = make_test_db_dir();

    let server = TkzrServer::test_server(&db_dir, port);
    let server_handle = server.start().expect("Failed to start server");
    

    let health_url = format!("http://localhost:{}/health", port);

    let res = reqwest::get(&health_url);
    assert!(res.is_ok(), "Error sending get request to /health: {}", res.unwrap_err());

    let response = res.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    server_handle.shutdown();
}

#[test]
/// Check that server responds with 405 to get requests on /sync/{example_replica_uuid}
fn test_sync_server_invalid_method() {
    let port = DEFAULT_PORT+4;

    let db_dir = make_test_db_dir();

    let server = TkzrServer::test_server(&db_dir, port);
    let server_handle = server.start().expect("Failed to start server");


    let client_replica_id = example_replica_1();
    let sync_url = format!("http://localhost:{}/sync/{}", port, client_replica_id);

    let client = reqwest::Client::new();
    let res = client.get(&sync_url)
        .send();
    assert!(res.is_ok(), "Error sending get request to /sync/{}: {}", res.unwrap_err(), client_replica_id);

    let response = res.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

    server_handle.shutdown();
}

#[test]
/// Check that server with empty database responds to sync query from new client replica with
/// empty JSON
fn test_sync_server_sync_empty() {
    let port = DEFAULT_PORT+5;

    let db_dir = make_test_db_dir();

    let server = TkzrServer::test_server(&db_dir, port);
    let server_handle = server.start().expect("Failed to start server");

    let client_replica_id = example_replica_1();
    let sync_url = format!("http://localhost:{}/sync/{}", port, client_replica_id);
    let ops: Vec<USetOp> = Vec::new();

    let client = reqwest::Client::new();
    let res = client.post(&sync_url)
        .json(&ops)
        .send();
    assert!(res.is_ok(), "Error sending post request to /sync/{}: {}", client_replica_id, res.unwrap_err());

    let mut response = res.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "body: {:?}",  response.text());

    let json_res = response.json();
    assert!(json_res.is_ok(), "Failed to parse response from server to json: {}", json_res.unwrap_err());

    let response_ops: Vec<USetOp> = json_res.unwrap();
    assert!(response_ops.is_empty(), "Unexpected USet ops returned from empty server to new client: {:?}", response_ops);

    server_handle.shutdown();
}

#[test]
/// Check that server with empty database responds to clear uset op query from new client replica
/// with 200
fn test_sync_server_clear_empty() {
    let port = DEFAULT_PORT+6;

    let db_dir = make_test_db_dir();

    let server = TkzrServer::test_server(&db_dir, port);
    let server_handle = server.start().expect("Failed to start server");

    let client_replica_id = example_replica_1();
    let sync_url = format!("http://localhost:{}/clear/{}", port, client_replica_id);

    let client = reqwest::Client::new();
    let res = client.post(&sync_url).send();
    assert!(res.is_ok(), "Error sending post request to /clear/{}: {}", client_replica_id, res.unwrap_err());

    let mut response = res.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "body: {:?}",  response.text());

    server_handle.shutdown();
}
