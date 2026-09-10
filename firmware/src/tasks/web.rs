use crate::web;

static CONFIG: picoserve::Config = picoserve::Config::const_default().keep_connection_alive();

/// The size of the task pool for the web server task.
///
/// At least 2 tasks are required for the web server to function correctly (one for SSE and one for handling regular HTTP requests).
pub const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static picoserve::AppRouter<web::App<'static>>,
    state: &'static web::AppState<'static>,
) {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 4096 * 2];

    picoserve::Server::new(&app.shared().with_state(state), &CONFIG, &mut http_buffer)
        .listen_and_serve(task_id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}
