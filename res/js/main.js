window.addEventListener('load', function () {
    let file_upload = document.getElementById("file-upload");
    file_upload.addEventListener("change", encrypt_file);
});

let exported_key = null;

function encrypt_file(){
    let file = this.files[0];
    let filename = file.name;
    let reader = new FileReader();

    reader.readAsArrayBuffer(file);

    reader.onload = async function (e) {
        let data = e.target.result;
        let file_iv = window.crypto.getRandomValues(new Uint8Array(12));
        let filename_iv = window.crypto.getRandomValues(new Uint8Array(12));

        let key = await window.crypto.subtle.generateKey({
                name: "AES-GCM",
                length: 128
            },
            true,
            ["encrypt", "decrypt"]
        );
        let encrypted_file = await window.crypto.subtle.encrypt(
            {
                name: "AES-GCM",
                iv: file_iv
            },
            key,
            data
        );
        console.log(filename);
        let encrypted_file_name = await window.crypto.subtle.encrypt(
            {
                name: "AES-GCM",
                iv: filename_iv
            },
            key,
            str2ab(filename)
        );

        exported_key = await exportCryptoKey(key);

        var ajax = new XMLHttpRequest;
        ajax.upload.addEventListener("progress", myProgressHandler, false);
        ajax.addEventListener('load', showQRCode, false);
        ajax.open('POST', '/upload', true);

        let upload = new FormData();
        upload.append("file", new Blob([encrypted_file]), arrayBufferToBase64(encrypted_file_name));
        upload.append("encrypted_filename", arrayBufferToBase64(encrypted_file_name));
        upload.append("file_iv", arrayBufferToBase64(file_iv));
        upload.append("filename_iv", arrayBufferToBase64(filename_iv));

        ajax.send(upload);
    };
}

async function exportCryptoKey(key) {
    const exported = await window.crypto.subtle.exportKey(
        "raw",
        key
    );

    return btoa(String.fromCharCode(...new Uint8Array(exported)));
}
function myProgressHandler(event) {
    var p = Math.floor(event.loaded/event.total*100);
    let progress_bar = document.getElementById("download-progress");
    let progress_div = document.getElementById("download-progress-div");
    if(progress_div.style.display === "none"){
        progress_div.style.display = "block";
        document.getElementById("file-picker").style.display = "none";
    }
    progress_bar.style.width = p+"%";
    progress_bar.innerText = p+"%";
    document.title = p+'%';
}

function showQRCode(event) {
    let file_id = event.target.responseText;
    let url = "http://127.0.0.1:8000/d/"+file_id+"#key="+exported_key;
    new QRCode(document.getElementById("qrcode"), {
        text: url,
        width: 128,
        height: 128,
        colorDark : "#000000",
        colorLight : "#ffffff",
        correctLevel : QRCode.CorrectLevel.H
    });
    document.getElementById("sharelink").value = url;
    document.getElementById("upload-success").style.display = "";
}

function str2ab(str) {
    var enc = new TextEncoder();
    return enc.encode(str);
}

function arrayBufferToBase64( buffer ) {
    var binary = '';
    var bytes = new Uint8Array( buffer );
    var len = bytes.byteLength;
    for (var i = 0; i < len; i++) {
        binary += String.fromCharCode( bytes[ i ] );
    }
    return window.btoa( binary );
}