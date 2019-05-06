use std::net::SocketAddr;

use std::thread::JoinHandle;

use failure::Error;

use actix_web::{HttpServer, App, web, HttpResponse};
use actix_web::dev::Server;

use crate::config::Config;
use crate::db::DBBackend;
use crate::sync::{USetOp, ReplicaUuid, apply_all_uset_ops};

// used in testing, but want the definitions to be next to each other
#[cfg(test)]
pub const DEFAULT_PORT: u16 = 24096;
// used for structopt/clap arguments because they only take &str
pub static DEFAULT_PORT_STR: &'static str = "24096";


// TODO: When https://github.com/actix/actix-net/pull/16 gets used in actix-web, we can derive
// Debug
//#[derive(Debug)]
pub struct TkzrServerHandle {
    server: Server,
    actix_thread_handle: JoinHandle<()>,
}

impl TkzrServerHandle {
    /// Run the server forever.
    pub fn join(self) -> std::thread::Result<()> {
        self.actix_thread_handle.join()
    }

    /// Shutdown the server.
    pub fn shutdown(&self) {
        self.server.stop(true);
    }
}

#[derive(Debug, Clone)]
pub struct TkzrServer {
    addr: SocketAddr,
    config: Config,
}

impl TkzrServer {
    /// Create a new `TkzrServer` that can be used to sync clients. Listens for incoming connections
    /// on `addr`.
    pub fn new(addr: impl Into<SocketAddr>, config: Config) -> TkzrServer {
        let addr = addr.into();
        TkzrServer {
            addr,
            config,
        }
    }

    #[cfg(test)]
    pub fn test_server(path: impl AsRef<std::path::Path>, port: u16) -> TkzrServer {
        let localhost = std::net::Ipv4Addr::new(127,0,0,1);
        let test_config = Config { db_path: path.as_ref().into(), break_cutoff: 0.0 };

        TkzrServer::new(SocketAddr::new(localhost.into(), port), test_config)
    }

    /// Start the server on a separate thread, returning a `TkzrServerHandle` that can be used to
    /// shutdown the server.
    pub fn start(self) -> Result<TkzrServerHandle, Error> {
        let (tx, rx) = std::sync::mpsc::channel();

        let addr = self.addr.clone();
        let config = self.config.clone();

        // So, this is a tiny bit complicated so here's an explanation:
        // We want to start the server on a separate thread so that it's easy to test, but we also
        // want to be able to shut it down, so we need a handle to the server somehow.
        //
        // I'm not 100% clear on how actix works (eg. how does HttpServer know which runtime to
        // use? I'm pretty sure it's stored in thread local storage, so per-thread but not 100%)
        // but we need the following things to happen in order:
        // - Create the actix runtime
        // - Create and bind an actix-web HttpServer
        // - Start/run the actix runtime, which then drives the HttpServer to accept connections
        //
        // Running the actix runtime/System blocks the thread on which it is started, so we have to
        // run it on a separate thread. However, we also want to get the Server returned by
        // HttpServer.start() so that we have the ability to shut it down in the current thread.
        //
        // We send it to the original thread via a mpsc::channel. At some point I was sending an
        // Arc<Mutex<Server>> via a sync_channel(0) so that it blocked, but I removed them and it
        // still works, so I'm not sure what problem I was having - maybe I was trying to send the
        // actual runtime over the channel accidentally.
        //
        // Finally, we take the thread handle from calling spawn, and put that in the
        // TkzrServerHandle (along with the actix Server) so that we can join on the thread and run
        // forever. It might be better to return them separately so that it's possible to test
        // "running forever" and "shutdown" at the same time.
        let actix_thread_handle = std::thread::spawn(move || {
            let actix_runtime = actix_rt::System::new("tkzr_server_runtime");
            let server_res = HttpServer::new( move || {
                App::new()
                    .service(web::resource("/health").to(|| HttpResponse::Ok()))
                    .data(config.clone())
                    .service(
                        web::resource("/sync/{replica_id}")
                             .route(web::post().to(handle_sync_route))
                     )
                    .service(
                        web::resource("/clear/{replica_id}")
                             .route(web::post().to(handle_clear_route))
                     )
            })
            .bind(addr)
            .map_err(|e| format_err!("Failed to bind to address {}: {}", addr, e));

            match server_res {
                Ok(server) => {
                    tx.send(Ok(server.start()))
                        .expect("failed to send server handle on startup");
                    actix_runtime.run()
                        .expect("actix runtime failed");
                }
                Err(e) => {
                    tx.send(Err(e))
                        .expect("Failed to send error on startup");
                }
            };
        });

        let server = rx.recv()?
            .map_err(|e| format_err!("Failed to acquire handle to server while starting up: {}", e))?;
        Ok(TkzrServerHandle {server, actix_thread_handle})
    }
}

fn handle_sync_route(config: web::Data<Config>, replica_id: web::Path<ReplicaUuid>, incoming_ops: web::Json<Vec<USetOp>>)
        -> Result<web::Json<Vec<USetOp>>, failure::Error> {
    let mut db = config.db()?;
    let tx = db.transaction()?;

    process_sync(tx, replica_id.into_inner(), &incoming_ops).map(|ops| web::Json(ops))
}

fn handle_clear_route(config: web::Data<Config>, replica_id: web::Path<ReplicaUuid>)
        -> Result<(), failure::Error> {
    let mut db = config.db()?;
    let tx = db.transaction()?;

    process_clear(tx, replica_id.into_inner())
}

/// Clear all pending USetOpMsgs for the replica. The client should call this method after a
/// successful sync.
fn process_clear(tx: impl DBBackend, replica_id: ReplicaUuid) -> Result<(), failure::Error> {
    tx.clear_uset_op_msgs(&replica_id)
        .map_err(|e| format_err!("Failed to clear USet ops for replica {} during incoming clear operation: {}", &replica_id, e))?;
    tx.finish()
}

/// Process incoming list of USet operations and return unsynced ops. If `replica_id` is unknown,
/// returns USetOp::Add messages for all tasks and adds `replica_id` to the replica set.
///
/// The pending USetOpMsgs in the database are not cleared - the client must call clear once it has
/// received all of the messages.
fn process_sync(tx: impl DBBackend, replica_id: ReplicaUuid, incoming_ops: &[USetOp]) -> Result<Vec<USetOp>, failure::Error> {
    apply_all_uset_ops(&tx, incoming_ops)
        .map_err(|e| format_err!("Failed to apply all uset ops during incoming sync operation: {}", e))?;

    let replicas = tx.fetch_replicas()
        .map_err(|e| format_err!("Failed to fetch replicas during incoming sync operation: {}", e))?;

    // if we already know about this replica, send it the updates it needs
    if replicas.iter().find(|&(r,_)| *r == replica_id).is_some() {
        let msgs = tx.fetch_uset_op_msgs(&replica_id)
            .map_err(|e| format_err!("Failed to fetch USet ops for replica {} during incoming sync operation: {}", &replica_id, e))?;

        tx.finish()?;
        // FIXME: I don't understand why the compiler complains if I don't use `return` here.
        return Ok(msgs.into_iter().map(|msg| msg.op).collect());
    }

    // otherwise, add it to the replica set and send it all the existing tasks
    {
        tx.store_replica_client(&replica_id)
            .map_err(|e| format_err!("Failed to store new client replica {} during incoming sync operation: {}", &replica_id, e))?;
        let tasks = tx.fetch_all_tasks()?;

        tx.finish()?;
        return Ok(tasks.into_iter().map(|t| USetOp::Add(t)).collect());
    }
}


#[cfg(test)]
mod test;
