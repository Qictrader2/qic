/* elm-pkg-js
port subscribeChatWS : String -> Cmd msg
port unsubscribeChatWS : String -> Cmd msg
port chatWsMessage : (String -> msg) -> Sub msg
*/

window.elmPkgJs = window.elmPkgJs || {};
window.elmPkgJs['chat-ws'] = {
    init: function(app) {
        if (!app.ports || !app.ports.subscribeChatWS || !app.ports.unsubscribeChatWS || !app.ports.chatWsMessage) {
            console.warn('elm-pkg-js [chat-ws]: required ports not found');
            return;
        }
        var connections = {};

        function sendToElm(data) {
            if (app.ports.chatWsMessage) {
                app.ports.chatWsMessage.send(data);
            }
        }

        function sendConnectionState(conversationId, state) {
            sendToElm(JSON.stringify({
                type: 'connection_state',
                conversation_id: conversationId,
                state: state
            }));
        }

        function wsUrl(conversationId, lastSeq) {
            var proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
            var url = proto + '//' + location.host + '/api/chat/ws/' + conversationId;
            if (lastSeq > 0) {
                url += '?last_seq=' + lastSeq;
            }
            return url;
        }

        function connect(conversationId) {
            // Skip if already connected or connecting
            if (connections[conversationId] && connections[conversationId].ws &&
                (connections[conversationId].ws.readyState === WebSocket.CONNECTING ||
                 connections[conversationId].ws.readyState === WebSocket.OPEN)) {
                return;
            }

            // Clean up any stale entry
            if (connections[conversationId]) {
                if (connections[conversationId].reconnectTimer) {
                    clearTimeout(connections[conversationId].reconnectTimer);
                }
                if (connections[conversationId].ws) {
                    connections[conversationId].ws.close();
                }
            }

            var lastSeq = (connections[conversationId] && connections[conversationId].lastSeq) || 0;
            var attempt = (connections[conversationId] && connections[conversationId].attempt) || 0;

            var ws = new WebSocket(wsUrl(conversationId, lastSeq));

            connections[conversationId] = {
                ws: ws,
                reconnectTimer: null,
                attempt: attempt,
                lastSeq: lastSeq,
                intentionalClose: false
            };

            ws.onopen = function() {
                connections[conversationId].attempt = 0;
            };

            ws.onmessage = function(event) {
                var data;
                try {
                    data = JSON.parse(event.data);
                } catch (e) {
                    console.warn('elm-pkg-js [chat-ws]: failed to parse WS message:', e.message);
                    return;
                }

                // Track sequence number for resume
                if (data.seq && data.seq > connections[conversationId].lastSeq) {
                    connections[conversationId].lastSeq = data.seq;
                }

                // The "connected" frame from the server
                if (data.type === 'connected') {
                    sendConnectionState(conversationId, 'connected');
                    return;
                }

                // All other events: forward the raw JSON to Elm.
                // The server sends {seq, type, ...fields} — Elm decodes by "type" field.
                sendToElm(event.data);
            };

            ws.onclose = function(event) {
                if (!connections[conversationId]) return;
                if (connections[conversationId].intentionalClose) return;

                var currentAttempt = connections[conversationId].attempt + 1;
                connections[conversationId].attempt = currentAttempt;
                connections[conversationId].ws = null;

                // Custom close code 4001 = lagged, always reconnect immediately
                var maxRetries = (event.code === 4001) ? Infinity : 30;

                if (currentAttempt > maxRetries) {
                    console.warn('elm-pkg-js [chat-ws]: max retries for', conversationId);
                    sendConnectionState(conversationId, 'disconnected');
                    delete connections[conversationId];
                    return;
                }

                var delay = (event.code === 4001) ? 100 :
                    Math.min(1000 * Math.pow(1.5, currentAttempt), 30000);
                sendConnectionState(conversationId, 'reconnecting');

                connections[conversationId].reconnectTimer = setTimeout(function() {
                    if (connections[conversationId]) {
                        connect(conversationId);
                    }
                }, delay);
            };

            ws.onerror = function() {
                // onclose will fire after onerror, so reconnection is handled there
            };
        }

        var MAX_CONNECTIONS = 4;

        function evictOldest(exceptId) {
            var ids = Object.keys(connections);
            if (ids.length < MAX_CONNECTIONS) return;
            for (var i = 0; i < ids.length; i++) {
                if (ids[i] !== exceptId) {
                    var conn = connections[ids[i]];
                    if (conn.reconnectTimer) clearTimeout(conn.reconnectTimer);
                    if (conn.ws) {
                        conn.intentionalClose = true;
                        conn.ws.close();
                    }
                    delete connections[ids[i]];
                    if (Object.keys(connections).length < MAX_CONNECTIONS) return;
                }
            }
        }

        app.ports.subscribeChatWS.subscribe(function(conversationId) {
            evictOldest(conversationId);
            connect(conversationId);
        });

        app.ports.unsubscribeChatWS.subscribe(function(conversationId) {
            var conn = connections[conversationId];
            if (conn) {
                if (conn.reconnectTimer) {
                    clearTimeout(conn.reconnectTimer);
                }
                if (conn.ws) {
                    conn.intentionalClose = true;
                    conn.ws.close();
                }
                delete connections[conversationId];
            }
        });
    }
};
