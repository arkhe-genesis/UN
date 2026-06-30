use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
};

pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if socket.send(msg).await.is_err() {
                break;
            }
        } else {
            break;
        }
    }
}
