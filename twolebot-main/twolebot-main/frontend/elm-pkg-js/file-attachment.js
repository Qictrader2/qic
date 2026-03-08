/* elm-pkg-js
port fileSelected : (D.Value -> msg) -> Sub msg
*/

window.elmPkgJs = window.elmPkgJs || {};
window.elmPkgJs['file-attachment'] = {
    init: function(app) {
        var fileInput = document.getElementById('chat-file-input');
        if (!fileInput) {
            fileInput = document.createElement('input');
            fileInput.type = 'file';
            fileInput.id = 'chat-file-input';
            fileInput.multiple = true;
            fileInput.style.cssText = 'position:absolute;width:0;height:0;overflow:hidden;opacity:0;';
            document.body.appendChild(fileInput);
        }

        function readAndSend(file) {
            var reader = new FileReader();
            reader.onloadend = function() {
                var base64 = reader.result.split(',')[1];
                if (app.ports.fileSelected) {
                    app.ports.fileSelected.send({
                        name: file.name,
                        mime_type: file.type || 'application/octet-stream',
                        data: base64
                    });
                }
            };
            reader.readAsDataURL(file);
        }

        fileInput.addEventListener('change', function(e) {
            var files = e.target.files;
            if (!files || files.length === 0) return;
            for (var i = 0; i < files.length; i++) {
                readAndSend(files[i]);
            }
            fileInput.value = '';
        });
    }
};
