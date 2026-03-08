/* elm-pkg-js
port scrollToBottom : String -> Cmd msg
port forceScrollToBottom : String -> Cmd msg
*/

window.elmPkgJs = window.elmPkgJs || {};
window.elmPkgJs['ui-helpers'] = {
    init: function(app) {
        // Track per-element scroll state
        var scrollState = {};
        var scrollPending = {};

        function ensureScrollTracking(el, elementId) {
            if (el._scrollTracked) return;
            el._scrollTracked = true;
            scrollState[elementId] = { isAtBottom: true };
            el.addEventListener('scroll', function() {
                var dist = el.scrollHeight - el.scrollTop - el.clientHeight;
                scrollState[elementId].isAtBottom = dist < 100;
            }, { passive: true });
        }

        // Smart scroll: only if user is at the bottom. Debounced per frame.
        if (app.ports && app.ports.scrollToBottom) {
            app.ports.scrollToBottom.subscribe(function(elementId) {
                if (scrollPending[elementId]) return;
                scrollPending[elementId] = true;
                requestAnimationFrame(function() {
                    scrollPending[elementId] = false;
                    var el = document.getElementById(elementId);
                    if (!el) return;
                    ensureScrollTracking(el, elementId);
                    if (scrollState[elementId].isAtBottom) {
                        el.scrollTop = el.scrollHeight;
                    }
                });
            });
        }

        // Force scroll: always goes to bottom (initial load, user sends message)
        if (app.ports && app.ports.forceScrollToBottom) {
            app.ports.forceScrollToBottom.subscribe(function(elementId) {
                requestAnimationFrame(function() {
                    requestAnimationFrame(function() {
                        var el = document.getElementById(elementId);
                        if (!el) return;
                        ensureScrollTracking(el, elementId);
                        el.scrollTop = el.scrollHeight;
                        scrollState[elementId].isAtBottom = true;
                    });
                });
            });
        }

        document.addEventListener('input', function(e) {
            if (e.target && e.target.id === 'chat-input') {
                e.target.style.height = 'auto';
                e.target.style.height = e.target.scrollHeight + 'px';
            }
        });

        document.addEventListener('click', function(e) {
            if (e.target && (e.target.id === 'chat-send-btn' || e.target.closest('#chat-send-btn'))) {
                requestAnimationFrame(function() {
                    var chatInput = document.getElementById('chat-input');
                    if (chatInput) chatInput.style.height = 'auto';
                });
            }
        });

        document.addEventListener('keydown', function(e) {
            if (e.target && e.target.id === 'chat-input') {
                var isSend = (e.key === 'Enter' && !e.shiftKey);
                if (isSend) {
                    e.preventDefault();
                    var sendBtn = document.getElementById('chat-send-btn');
                    if (sendBtn && !sendBtn.disabled) {
                        sendBtn.click();
                        requestAnimationFrame(function() {
                            e.target.style.height = 'auto';
                        });
                    }
                }
            }
        });
    }
};
