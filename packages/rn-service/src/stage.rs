use std::future::Future;
use std::net::SocketAddr;

use axum::Router;

use crate::http::serve_http;
use crate::signal::shutdown_signal;
use crate::traits::{BoxFuture, RpcServer};

type WorkerTask = BoxFuture<Result<(), String>>;
type HttpTask = BoxFuture<(Router, u16)>;
type RpcTask = BoxFuture<Result<(), Box<dyn std::error::Error>>>;

/// Stage where application state is ready, allowing workers, HTTP, and RPC servers to be registered.
pub struct RNServiceStateStage<S> {
    runtime: tokio::runtime::Runtime,
    state: S,
    service_name: Option<String>,
    workers: Vec<WorkerTask>,
    http_task: Option<HttpTask>,
    rpc_task: Option<RpcTask>,
}

impl<S: Clone + Send + Sync + 'static> RNServiceStateStage<S> {
    /// Creates a new state stage instance.
    pub fn new(runtime: tokio::runtime::Runtime, state: S, service_name: Option<String>) -> Self {
        Self {
            runtime,
            state,
            service_name,
            workers: Vec::new(),
            http_task: None,
            rpc_task: None,
        }
    }

    /// Registers an asynchronous background worker task (e.g. RabbitMQ consumers, cron jobs).
    pub fn build_worker<F, Fut, E>(mut self, f: F) -> Self
    where
        E: std::fmt::Display + 'static,
        F: FnOnce(S) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
    {
        let state = self.state.clone();
        self.workers.push(Box::pin(async move {
            f(state).await.map_err(|e| e.to_string())
        }));
        self
    }

    /// Registers an Axum HTTP router and its listening port.
    pub fn with_http<F, Fut>(mut self, f: F) -> Self
    where
        F: FnOnce(S) -> Fut + Send + 'static,
        Fut: Future<Output = (Router, u16)> + Send + 'static,
    {
        let state = self.state.clone();
        self.http_task = Some(Box::pin(f(state)));
        self
    }

    /// Registers an RPC server returning `(server, port)`.
    pub fn with_rpc<F, Fut, R>(mut self, f: F) -> Self
    where
        R: RpcServer + 'static,
        F: FnOnce(S) -> Fut + Send + 'static,
        Fut: Future<Output = (R, u16)> + Send + 'static,
    {
        let state = self.state.clone();
        self.rpc_task = Some(Box::pin(async move {
            let (server, port) = f(state).await;
            server.serve(port).await
        }));
        self
    }

    /// Runs all registered workers and servers with coordinated graceful shutdown.
    pub fn run(mut self) -> ! {
        let _ = self.state;
        let workers = std::mem::take(&mut self.workers);
        let http_task = self.http_task.take();
        let rpc_task = self.rpc_task.take();
        let service_name = self.service_name.clone();

        self.runtime.block_on(async move {
            // 1. Run all registered worker initialization tasks
            for worker in workers {
                if let Err(e) = worker.await {
                    tracing::error!("Worker initialization failure: {e}");
                    std::process::exit(1);
                }
            }

            // 2. Start servers
            let http_fut = async {
                if let Some(task) = http_task {
                    let (router, port) = task.await;
                    let addr = SocketAddr::from(([0, 0, 0, 0], port));
                    if let Err(e) = serve_http(addr, router, shutdown_signal()).await {
                        tracing::error!("HTTP server fatal error: {e}");
                        std::process::exit(1);
                    }
                } else {
                    std::future::pending::<()>().await;
                }
            };

            let rpc_fut = async {
                if let Some(task) = rpc_task {
                    tokio::select! {
                        res = task => {
                            if let Err(e) = res {
                                tracing::error!("RPC server error: {e}");
                            }
                        }
                        _ = shutdown_signal() => {
                            tracing::info!("RPC server shutting down");
                        }
                    }
                } else {
                    std::future::pending::<()>().await;
                }
            };

            // If only workers were configured with no servers, run until shutdown
            let worker_only_guard = async {
                shutdown_signal().await;
            };

            tokio::select! {
                _ = http_fut => {},
                _ = rpc_fut => {},
                _ = worker_only_guard => {},
            }

            if let Some(name) = service_name {
                tracing::info!("{name} stopped gracefully");
            } else {
                tracing::info!("Service stopped gracefully");
            }

            std::process::exit(0);
        })
    }
}
