$(document).ready(function(){
    $('form').on('submit', function(e){
        e.preventDefault();

        var totalSize = 0;
        var formData = new FormData();
        var files = $('input[type=file]')[0].files;
        for (var i=0; i<files.length; i++) {
            var fileName = files[i].name;
            var fileExtension = fileName.slice((fileName.lastIndexOf(".") - 1 >>> 0) + 2);  // get file extension

            if (fileName.length > 100) {
                // alert("File name cannot exceed 100 characters");
                showNotification("danger", "Nazwa pliku nie może przekraczać 100 znaków");
                return;
            }

            if (!['stl', '3mf', 'obj'].includes(fileExtension.toLowerCase())) {
                // alert("Invalid file type. Only .stl, .3mf, .obj files are allowed");
                showNotification("danger", "Nieprawidłowy typ pliku. Dozwolone są tylko pliki .stl, .3mf, .obj");
                return;
            }

            totalSize += files[i].size;  // size in bytes
            formData.append('files', files[i]);
        }

        if (totalSize > 125 * 1024 * 1024) {  // 125MB in bytes
            // alert("Total file size exceeds 125MB");
            showNotification("danger", "Całkowity rozmiar plików przekracza 125 MB");
            return;
        }

        $.ajax({
            url : '/uploadfiles/',
            type : 'POST',
            data : formData,
            processData: false,  // tell jQuery not to process the data
            contentType: false,  // tell jQuery not to set contentType

            beforeSend: function() {
                $("#progressBarContainer").html('<div class="progress"><div class="progress-bar progress-bar-striped progress-bar-animated" role="progressbar" aria-valuenow="0" aria-valuemin="0" aria-valuemax="100" style="width: 0%"></div></div>');
            },
            xhr: function() {
                var xhr = new window.XMLHttpRequest();
                xhr.upload.addEventListener("progress", function(evt) {
                    if (evt.lengthComputable) {
                        var percentComplete = ((evt.loaded / evt.total) * 100);
                        console.log("Progress: " + percentComplete);
                        $(".progress-bar").css('width', percentComplete+'%').attr('aria-valuenow', percentComplete);
                    }
               }, false);
               return xhr;
            },
            success : function(data) {
                console.log(data);
                updateTable(data);
                $("#progressBarContainer").html('');
                showNotification("success", "Przesyłanie zakończone sukcesem!");
            },
            error: function(jqXHR) {
                $("#progressBarContainer").html('');
                if(jqXHR.status >= 400 && jqXHR.status < 600){
                    // alert('Something went wrong, error: ' + jqXHR.status);
                    showNotification("danger", "Error: " + jqXHR.status);
                }
            }
        });
    });
});

function showNotification(type, message) {
    var alertDiv = $('<div class="alert alert-' + type + ' alert-dismissible fade show" role="alert" style="position: fixed; bottom: 20px; right: 20px; z-index: 9999;">' +
        message +
        '<button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>' +
        '<div class="progress-bar-container" style="height: 4px; background-color: rgba(255, 255, 255, 0.5);"><div class="progress-bar-inner" style="height: 4px; width: 0%; background-color: #ffffff;"></div></div>' +
        '</div>');

    // Dodanie powiadomienia do DOM
    $("body").append(alertDiv);

    // Animacja paska postępu
    $('.progress-bar-inner', alertDiv).animate({ width: '100%' }, 5000);

    // Usunięcie powiadomienia po 5 sekundach
    setTimeout(function() {
        alertDiv.alert('close');
    }, 5000);
}
