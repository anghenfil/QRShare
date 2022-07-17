window.addEventListener('load', async function () {
    let hash = window.atob(window.location.hash.replace("#key=", ""));
    let file_id = window.location.pathname.replace("/d/", "");

    let metadata = await load_metadata(file_id);
    let key = await window.crypto.subtle.importKey(
        "raw",
        str2ab(hash),
        "AES-GCM",
        true,
        ["encrypt", "decrypt"]
    );

    //Decrypt filename:
    let decrypted_filename = await window.crypto.subtle.decrypt(
        {
            name: "AES-GCM",
            iv: base64ToArrayBuffer(metadata.filename_iv)
        },
        key,
        base64ToArrayBuffer(metadata.file_name)
    );
    let filename = ab2str(decrypted_filename);
    document.getElementById("encrypted_file_name").textContent = filename;

    document.getElementById("download_button").onclick = function(){
        load_file(file_id, key, base64ToArrayBuffer(metadata.file_iv), filename);
    }
});

async function load_file(file_id, key, iv, filename){
    const xhr = new XMLHttpRequest();
    xhr.responseType = "arraybuffer";
    xhr.onload = async () => {
        let decrypted = await window.crypto.subtle.decrypt(
            {
                name: "AES-GCM",
                iv: iv,
            },
            key,
            xhr.response
        );
        let link = document.createElement('a');
        link.download = filename;
        let blob = new Blob([decrypted], {});
        link.href = URL.createObjectURL(blob);
        link.click();
    };

    xhr.onerror = () => {
        alert('Download failed.');
    }

    xhr.onabort = () => {
        alert('Download cancelled.');
    }

    xhr.onprogress = (event) => {
        let p = Math.floor(event.loaded/event.total*100);
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

    xhr.open('GET', '/download/'+file_id, true);
    xhr.send();
}

async function load_metadata(file_id) {
    let request = await fetch("/metadata/" + file_id);
    if (request.ok){
        let fm = await request.json();
        return fm;
    }else{
        alert("Couldn't find requested file.");
    }
}

/*
Convert a string into an ArrayBuffer
from https://developers.google.com/web/updates/2012/06/How-to-convert-ArrayBuffer-to-and-from-String
*/
function str2ab(str) {
    const buf = new ArrayBuffer(str.length);
    const bufView = new Uint8Array(buf);
    for (let i = 0, strLen = str.length; i < strLen; i++) {
        bufView[i] = str.charCodeAt(i);
    }
    return buf;
}
function ab2str(buf) {
    var enc = new TextDecoder("utf-8");
    return enc.decode(buf);
}
function base64ToArrayBuffer(base64) {
    var binary_string =  window.atob(base64);
    var len = binary_string.length;
    var bytes = new Uint8Array( len );
    for (var i = 0; i < len; i++)        {
        bytes[i] = binary_string.charCodeAt(i);
    }
    return bytes.buffer;
}